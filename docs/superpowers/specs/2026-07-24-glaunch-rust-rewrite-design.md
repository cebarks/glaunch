# glaunch Rust Rewrite — Design Spec

## Overview

Rewrite the `glaunch` bash script as a Rust CLI application. The current script wraps Steam game launches with gamescope, gamemode, HDR, V-Cache pinning, vkBasalt, and FSR4. The Rust version retains all existing functionality and adds: per-game profiles, MangoHud integration, a TUI profile editor, dry-run/verbose modes, and hardware detection reporting.

## Architecture

**Approach**: Monolithic single-binary CLI using `clap` (derive) for arg parsing and `ratatui`/`crossterm` for the TUI. No workspace or feature-flag splitting.

**Key crates**:
- `clap` (derive) — CLI arg parsing, subcommands, shell completions
- `serde` + `toml` — config serialization
- `ratatui` + `crossterm` — terminal UI
- `dirs` — XDG directory resolution
- `anyhow` — application error handling

## CLI Subcommand Structure

```
glaunch run [flags] -- %command%    # Launch a game (drop-in replacement)
glaunch profile <subcommand>        # Manage per-game profiles
glaunch tui                         # Interactive profile editor
glaunch info                        # Show detected hardware
```

### `glaunch run`

The hot path — called from Steam launch options. Must be fast.

**Flags (carried over from bash)**:
- `--itm` — Enable HDR inverse tone mapping (for SDR games)
- `--fsr4` — Enable FSR4 RDNA3 upgrade
- `--no-hdr` — Disable HDR entirely
- `--no-vrr` — Disable adaptive sync
- `--no-gamescope` — Skip gamescope, just gamemoderun + env
- `--vcache` — Pin to V-Cache CCD (auto-detects X3D topology)
- `--no-vcache` — Disable V-Cache pinning
- `--fix-mouse` — Fix Steam mouse stutter (LD_PRELOAD gamemode)
- `--vkbasalt [PROFILE]` — Enable vkBasalt with optional named profile
- `-w, --width <W>` — Override width (default: 3840)
- `-h, --height <H>` — Override height (default: 2160)

**New flags**:
- `--dry-run` / `-n` — Print the final command instead of executing it
- `--verbose` / `-v` — Log each decision to stderr
- `--profile <name|appid>` — Load a saved profile
- `--mangohud [CONFIG]` — Enable MangoHud with optional config preset

**Override semantics**: CLI flags > profile settings > global defaults > hardcoded defaults. Unset fields in any layer fall through to the next.

**App ID auto-detection**: When called from Steam (`%command%`), parse the app ID from the command arguments. Steam passes commands like `/home/user/.steam/steam/steamapps/common/GameName/game.exe` — extract the app ID by reading the corresponding `appmanifest_*.acf` file from the steamapps directory, or by matching against `steam_app_id` fields in existing profiles. If no profile matches, proceed with global defaults (no error).

### `glaunch profile`

- `profile list` — Show all profiles in a table
- `profile show <name>` — Print a profile's full settings
- `profile create <name>` — Create a new profile (from flags or interactive)
- `profile edit <name>` — Open profile in `$EDITOR`
- `profile delete <name>` — Delete a profile (with confirmation)

### `glaunch tui`

Interactive ratatui-based profile management tool. Does NOT launch games.

### `glaunch info`

Print detected hardware: V-Cache CCD topology, display resolution, GPU, refresh rate.
- `--refresh` flag to force re-detection (bypasses cache)

## Configuration

### Directory Layout

All config lives under XDG: `~/.config/glaunch/`

```
~/.config/glaunch/
├── config.toml              # Global defaults
├── hardware_cache.toml      # Cached hardware detection
└── profiles/
    ├── elden-ring.toml
    ├── rdr2.toml
    └── ...
```

### Global Config (`config.toml`)

```toml
[defaults]
width = 3840
height = 2160
hdr = true
vrr = true
gamescope = true

[mangohud]
enabled = false

[hardware]
# Auto-detected, but overridable
# vcache_cpus = "0-7"
```

### Per-Game Profile (`profiles/<slug>.toml`)

```toml
name = "Elden Ring"
steam_app_id = 1245620

[settings]
hdr = true
itm = true
vcache = true
fsr4 = false
width = 3840
height = 2160

[mangohud]
enabled = true

[vkbasalt]
enabled = true
profile = "reshade-film"
```

Profile fields are all optional — only set what differs from global defaults. The slug (filename) is the canonical identifier; `steam_app_id` enables auto-matching.

## Hardware Detection

### V-Cache CCD Detection

Reads `/sys/devices/system/cpu/cpu*/cache/index3` to find asymmetric L3 sizes, identifying the V-Cache CCD. Logic:

1. Scan all index3 cache entries, record size and `shared_cpu_list` for each
2. Find the largest L3 — if all CCDs have equal L3, there's no X3D topology
3. If asymmetric, the largest-L3 CCD is the V-Cache CCD

**Caching**: Detection result written to `~/.config/glaunch/hardware_cache.toml`. Re-detected on `glaunch info --refresh` or when cache is >30 days old.

**Error handling**: If `--vcache` is requested but no asymmetric L3 is found, print a clear diagnostic ("No X3D V-Cache topology detected — all CCDs have equal L3") and proceed without pinning (non-fatal).

### Display/GPU Detection

Read from `/sys/class/drm/` for `glaunch info` output. Not used in the launch path — purely informational.

## MangoHud Integration

MangoHud is controlled via environment variables:

- `MANGOHUD=1` — enables the overlay
- `MANGOHUD_CONFIG=<path>` — optional per-game config file

glaunch sets these env vars when MangoHud is enabled (via `--mangohud`, profile, or global config). Config file resolution follows the same pattern as vkBasalt: `~/.config/MangoHud/<name>.conf`.

## Process Execution

The `run` subcommand builds a command and replaces itself via `exec` (Rust's `CommandExt::exec` on Unix). No child process — signals pass through correctly.

**Execution chain**:
1. Resolve settings (CLI flags → profile → global defaults → hardcoded)
2. Set environment variables: `PROTON_FSR4_RDNA3_UPGRADE`, `ENABLE_VKBASALT`, `VKBASALT_CONFIG_FILE`, `MANGOHUD`, `MANGOHUD_CONFIG`, `LD_PRELOAD` (mouse fix)
3. Build command:
   - If gamescope: `gamemoderun gamescope <args> -- <game command>`
   - If no gamescope: `gamemoderun <game command>`
4. If V-Cache pinning: wrap with `systemd-run --user --scope -p AllowedCPUs=<cpus> --`
5. `exec` the final command

**Dry-run**: Prints the full command with env vars to stdout, then exits.
**Verbose**: Logs each decision to stderr with source attribution ("HDR: enabled (from profile 'elden-ring')").

## TUI Design

Built with `ratatui` + `crossterm`. Launched via `glaunch tui`.

### Views

**Profile List** (default):
- Table of all profiles: name, app ID, key settings (HDR, VRR, gamescope, vcache, mangohud)
- Status bar showing detected hardware

**Profile Editor**:
- Form with toggle fields (HDR, ITM, FSR4, VRR, gamescope, vcache, mangohud, vkbasalt)
- Text input fields (name, app ID, width, height, vkbasalt profile name)
- Save/cancel actions

### Navigation

Vim-style keybindings:
- `j`/`k` or arrows — navigate list
- `e` or `Enter` — edit selected profile
- `n` — new profile
- `d` — delete (with confirmation)
- `/` — search/filter
- `q` or `Esc` — quit / back

### Extensibility

Designed so a "game library" view can be added later (reading from Steam's `libraryfolders.vdf` and `appmanifest_*.acf` files).

## Error Handling

Application-level errors use `anyhow` for context chains. The `run` subcommand should fail loudly on configuration errors (missing vkBasalt profile, invalid resolution) but gracefully on hardware detection issues (no V-Cache found → warn and continue).

Exit codes:
- `0` — success (or dry-run)
- `1` — configuration/argument error
- `2` — hardware detection failure (fatal, e.g., can't read sysfs)

## Testing Strategy

- **Unit tests**: Config parsing, settings resolution/layering, V-Cache detection logic (with mock sysfs data), command building
- **Integration tests**: End-to-end dry-run tests that verify the correct command is produced for given inputs
- **No TUI tests initially**: TUI testing is complex; manual verification is sufficient for the initial version

## Future Considerations (Not In Scope)

- Steam library browsing in TUI
- Game-specific environment variable overrides in profiles
- Wayland/X11 compositor detection
- Profile import/export
- Shell completion generation (`clap_complete`)
