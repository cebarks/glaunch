use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::App;
use super::View;
use crate::config::{self, MangoHudConfig, Profile, ProfileSettings, VkBasaltConfig};

#[derive(Debug, Clone)]
pub struct EditState {
    pub is_new: bool,
    pub fields: Vec<Field>,
    pub selected: usize,
    pub editing_text: bool,
}

#[derive(Debug, Clone)]
pub enum Field {
    Text { label: String, value: String },
    Toggle { label: String, value: bool },
}

impl EditState {
    pub fn new_profile() -> Self {
        Self {
            is_new: true,
            fields: vec![
                Field::Text {
                    label: "Slug".into(),
                    value: String::new(),
                },
                Field::Text {
                    label: "Name".into(),
                    value: String::new(),
                },
                Field::Text {
                    label: "Steam App ID".into(),
                    value: String::new(),
                },
                Field::Text {
                    label: "Width".into(),
                    value: "3840".into(),
                },
                Field::Text {
                    label: "Height".into(),
                    value: "2160".into(),
                },
                Field::Toggle {
                    label: "HDR".into(),
                    value: true,
                },
                Field::Toggle {
                    label: "ITM".into(),
                    value: false,
                },
                Field::Toggle {
                    label: "VRR".into(),
                    value: true,
                },
                Field::Toggle {
                    label: "FSR4".into(),
                    value: false,
                },
                Field::Toggle {
                    label: "Gamescope".into(),
                    value: true,
                },
                Field::Toggle {
                    label: "V-Cache".into(),
                    value: false,
                },
                Field::Toggle {
                    label: "Fix Mouse".into(),
                    value: false,
                },
                Field::Toggle {
                    label: "MangoHud".into(),
                    value: false,
                },
                Field::Toggle {
                    label: "vkBasalt".into(),
                    value: false,
                },
                Field::Text {
                    label: "vkBasalt Profile".into(),
                    value: String::new(),
                },
            ],
            selected: 0,
            editing_text: false,
        }
    }

    pub fn from_profile(slug: String, profile: Profile, is_new: bool) -> Self {
        let s = profile.settings.as_ref();
        let m = profile.mangohud.as_ref();
        let v = profile.vkbasalt.as_ref();

        Self {
            is_new,
            fields: vec![
                Field::Text {
                    label: "Slug".into(),
                    value: slug,
                },
                Field::Text {
                    label: "Name".into(),
                    value: profile.name.unwrap_or_default(),
                },
                Field::Text {
                    label: "Steam App ID".into(),
                    value: profile
                        .steam_app_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                },
                Field::Text {
                    label: "Width".into(),
                    value: s
                        .and_then(|s| s.width)
                        .map(|w| w.to_string())
                        .unwrap_or_default(),
                },
                Field::Text {
                    label: "Height".into(),
                    value: s
                        .and_then(|s| s.height)
                        .map(|h| h.to_string())
                        .unwrap_or_default(),
                },
                Field::Toggle {
                    label: "HDR".into(),
                    value: s.and_then(|s| s.hdr).unwrap_or(true),
                },
                Field::Toggle {
                    label: "ITM".into(),
                    value: s.and_then(|s| s.itm).unwrap_or(false),
                },
                Field::Toggle {
                    label: "VRR".into(),
                    value: s.and_then(|s| s.vrr).unwrap_or(true),
                },
                Field::Toggle {
                    label: "FSR4".into(),
                    value: s.and_then(|s| s.fsr4).unwrap_or(false),
                },
                Field::Toggle {
                    label: "Gamescope".into(),
                    value: s.and_then(|s| s.gamescope).unwrap_or(true),
                },
                Field::Toggle {
                    label: "V-Cache".into(),
                    value: s.and_then(|s| s.vcache).unwrap_or(false),
                },
                Field::Toggle {
                    label: "Fix Mouse".into(),
                    value: s.and_then(|s| s.fix_mouse).unwrap_or(false),
                },
                Field::Toggle {
                    label: "MangoHud".into(),
                    value: m.and_then(|m| m.enabled).unwrap_or(false),
                },
                Field::Toggle {
                    label: "vkBasalt".into(),
                    value: v.and_then(|v| v.enabled).unwrap_or(false),
                },
                Field::Text {
                    label: "vkBasalt Profile".into(),
                    value: v.and_then(|v| v.profile.clone()).unwrap_or_default(),
                },
            ],
            selected: 0,
            editing_text: false,
        }
    }

    fn to_profile(&self) -> (String, Profile) {
        let field_val = |label: &str| -> String {
            self.fields
                .iter()
                .find_map(|f| match f {
                    Field::Text { label: l, value } if l == label => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };
        let field_bool = |label: &str| -> bool {
            self.fields
                .iter()
                .find_map(|f| match f {
                    Field::Toggle { label: l, value } if l == label => Some(*value),
                    _ => None,
                })
                .unwrap_or(false)
        };

        let slug = field_val("Slug");
        let name = field_val("Name");
        let app_id_str = field_val("Steam App ID");
        let width_str = field_val("Width");
        let height_str = field_val("Height");
        let vkbasalt_profile = field_val("vkBasalt Profile");

        let profile = Profile {
            name: if name.is_empty() { None } else { Some(name) },
            steam_app_id: app_id_str.parse().ok(),
            settings: Some(ProfileSettings {
                width: width_str.parse().ok(),
                height: height_str.parse().ok(),
                hdr: Some(field_bool("HDR")),
                vrr: Some(field_bool("VRR")),
                gamescope: Some(field_bool("Gamescope")),
                itm: Some(field_bool("ITM")),
                fsr4: Some(field_bool("FSR4")),
                vcache: Some(field_bool("V-Cache")),
                fix_mouse: Some(field_bool("Fix Mouse")),
            }),
            mangohud: Some(MangoHudConfig {
                enabled: Some(field_bool("MangoHud")),
                config: None,
            }),
            vkbasalt: Some(VkBasaltConfig {
                enabled: Some(field_bool("vkBasalt")),
                profile: if vkbasalt_profile.is_empty() {
                    None
                } else {
                    Some(vkbasalt_profile)
                },
            }),
        };

        (slug, profile)
    }
}

pub fn render(frame: &mut Frame, state: &EditState) {
    let title = if state.is_new {
        " New Profile "
    } else {
        " Edit Profile "
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(frame.area());
    frame.render_widget(block, frame.area());

    let field_count = state.fields.len();
    let constraints: Vec<Constraint> = (0..field_count + 1)
        .map(|_| Constraint::Length(1))
        .collect();

    let chunks = Layout::vertical(constraints).split(inner);

    for (i, field) in state.fields.iter().enumerate() {
        let is_selected = i == state.selected;
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let text = match field {
            Field::Text { label, value } => {
                let cursor = if is_selected && state.editing_text {
                    "|"
                } else {
                    ""
                };
                format!("  {label:<18} {value}{cursor}")
            }
            Field::Toggle { label, value } => {
                let indicator = if *value { "[x]" } else { "[ ]" };
                format!("  {label:<18} {indicator}")
            }
        };

        if i < chunks.len() {
            frame.render_widget(Paragraph::new(text).style(style), chunks[i]);
        }
    }

    // Help text at the bottom
    let help_idx = field_count;
    if help_idx < chunks.len() {
        let help = if state.editing_text {
            "  Enter: confirm | Esc: cancel"
        } else {
            "  j/k: navigate | Enter/Space: edit/toggle | s: save | Esc: cancel"
        };
        frame.render_widget(
            Paragraph::new(help).style(Style::default().fg(Color::DarkGray)),
            chunks[help_idx],
        );
    }
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    let edit_state = match &mut app.edit_state {
        Some(s) => s,
        None => return,
    };

    if edit_state.editing_text {
        match key.code {
            KeyCode::Esc => {
                edit_state.editing_text = false;
            }
            KeyCode::Enter => {
                edit_state.editing_text = false;
            }
            KeyCode::Backspace => {
                if let Field::Text { value, .. } = &mut edit_state.fields[edit_state.selected] {
                    value.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Field::Text { value, .. } = &mut edit_state.fields[edit_state.selected] {
                    value.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.edit_state = None;
            app.view = View::List;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let count = edit_state.fields.len();
            edit_state.selected = (edit_state.selected + 1) % count;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let count = edit_state.fields.len();
            edit_state.selected = if edit_state.selected == 0 {
                count - 1
            } else {
                edit_state.selected - 1
            };
        }
        KeyCode::Enter | KeyCode::Char(' ') => match &mut edit_state.fields[edit_state.selected] {
            Field::Toggle { value, .. } => *value = !*value,
            Field::Text { .. } => edit_state.editing_text = true,
        },
        KeyCode::Char('s') => {
            let (slug, profile) = edit_state.to_profile();
            if slug.is_empty() {
                app.status_message = Some("Slug cannot be empty".to_string());
                return;
            }
            match config::save_profile(&slug, &profile) {
                Ok(()) => {
                    app.status_message = Some(format!("Saved profile '{slug}'"));
                    let _ = app.reload_profiles();
                    app.edit_state = None;
                    app.view = View::List;
                }
                Err(e) => {
                    app.status_message = Some(format!("Save failed: {e}"));
                }
            }
        }
        _ => {}
    }
}
