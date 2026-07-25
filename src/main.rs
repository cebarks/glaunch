mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};
use glaunch::{config, hardware, launch};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run(args) => {
            let global = config::load_global_config()?;

            // Load profile (explicit flag, or auto-detect from Steam app ID)
            let profile = if let Some(ref profile_name) = args.profile {
                config::load_profile(profile_name)?
            } else if let Some(app_id) = config::extract_steam_app_id(&args.command) {
                if args.verbose {
                    eprintln!("glaunch: detected Steam app ID {app_id}");
                }
                config::find_profile_by_app_id(app_id)?.map(|(_, p)| p)
            } else {
                None
            };

            let cli_overrides = cli_overrides_from_run_args(&args);
            let settings = config::resolve_settings_from_layers(
                Some(&cli_overrides),
                profile.as_ref(),
                &global,
            );

            if args.verbose {
                log_settings(&settings, profile.as_ref());
            }

            // V-Cache detection (only if settings say vcache is enabled)
            let vcache = if settings.vcache {
                match hardware::detect_vcache() {
                    Ok(Some(info)) => {
                        if args.verbose {
                            eprintln!(
                                "glaunch: V-Cache detected — pinning to CPUs {} (L3: {}MB)",
                                info.cpus,
                                info.l3_size_kb / 1024
                            );
                        }
                        Some(info)
                    }
                    Ok(None) => {
                        eprintln!(
                            "glaunch: No X3D V-Cache topology detected — all CCDs have equal L3"
                        );
                        None
                    }
                    Err(e) => {
                        eprintln!("glaunch: V-Cache detection failed: {e}");
                        None
                    }
                }
            } else {
                None
            };

            let plan = launch::build_launch_plan(&settings, vcache.as_ref(), &args.command)?;

            if args.dry_run {
                println!("{}", launch::dry_run_output(&plan));
                return Ok(());
            }

            launch::execute(&plan)?;
            unreachable!()
        }
        Command::Profile(_args) => {
            eprintln!("profile: not yet implemented");
            std::process::exit(1);
        }
        Command::Tui => {
            eprintln!("tui: not yet implemented");
            std::process::exit(1);
        }
        Command::Info(args) => {
            let info = hardware::detect_hardware(args.refresh)?;

            println!("=== glaunch hardware info ===\n");

            match &info.vcache {
                Some(vc) => {
                    println!("V-Cache: detected");
                    println!("  CPUs: {}", vc.cpus);
                    println!("  L3 cache: {} MB", vc.l3_size_kb / 1024);
                }
                None => println!("V-Cache: not detected (symmetric L3 or single CCD)"),
            }

            if info.gpus.is_empty() {
                println!("\nGPU: not detected");
            } else {
                for (i, gpu) in info.gpus.iter().enumerate() {
                    println!("\nGPU {i}: {}", gpu.name);
                    if !gpu.driver.is_empty() {
                        println!("  Driver: {}", gpu.driver);
                    }
                }
            }

            Ok(())
        }
    }
}

fn cli_overrides_from_run_args(args: &cli::RunArgs) -> config::CliOverrides {
    config::CliOverrides {
        width: args.width,
        height: args.height,
        hdr: if args.no_hdr { Some(false) } else { None },
        vrr: if args.no_vrr { Some(false) } else { None },
        gamescope: if args.no_gamescope { Some(false) } else { None },
        itm: if args.itm { Some(true) } else { None },
        fsr4: if args.fsr4 { Some(true) } else { None },
        vcache: if args.vcache {
            Some(true)
        } else if args.no_vcache {
            Some(false)
        } else {
            None
        },
        fix_mouse: if args.fix_mouse { Some(true) } else { None },
        mangohud: args.mangohud.as_ref().map(|_| true),
        mangohud_config: args.mangohud.clone().filter(|s| !s.is_empty()),
        vkbasalt: args.vkbasalt.as_ref().map(|_| true),
        vkbasalt_profile: args.vkbasalt.clone().filter(|s| !s.is_empty()),
    }
}

fn log_settings(settings: &config::ResolvedSettings, profile: Option<&config::Profile>) {
    let src = |from_profile: bool| {
        if from_profile {
            profile
                .and_then(|p| p.name.as_deref())
                .map(|n| format!(" (from profile '{n}')"))
                .unwrap_or_default()
        } else {
            String::new()
        }
    };

    eprintln!(
        "glaunch: Resolution: {}x{}",
        settings.width, settings.height
    );
    eprintln!(
        "glaunch: HDR: {}{}",
        if settings.hdr { "enabled" } else { "disabled" },
        src(profile.is_some())
    );
    eprintln!(
        "glaunch: VRR: {}",
        if settings.vrr { "enabled" } else { "disabled" }
    );
    eprintln!(
        "glaunch: Gamescope: {}",
        if settings.gamescope {
            "enabled"
        } else {
            "disabled"
        }
    );
    if settings.itm {
        eprintln!("glaunch: ITM: enabled{}", src(profile.is_some()));
    }
    if settings.fsr4 {
        eprintln!("glaunch: FSR4: enabled");
    }
    if settings.mangohud {
        eprintln!("glaunch: MangoHud: enabled");
    }
    if settings.vkbasalt {
        let profile_info = settings.vkbasalt_profile.as_deref().unwrap_or("default");
        eprintln!("glaunch: vkBasalt: enabled (profile: {profile_info})");
    }
    if settings.fix_mouse {
        eprintln!("glaunch: Mouse fix: enabled");
    }
    if settings.vcache {
        eprintln!("glaunch: V-Cache pinning: requested");
    }
}
