use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};

use super::{App, View};
use crate::config;
use crate::tui::profile_edit;

#[derive(Debug, Clone)]
pub struct ListState {
    pub table_state: TableState,
    pub filter: String,
    pub filtering: bool,
    pub confirm_delete: bool,
}

impl ListState {
    pub fn new(count: usize) -> Self {
        let mut table_state = TableState::default();
        if count > 0 {
            table_state.select(Some(0));
        }
        Self {
            table_state,
            filter: String::new(),
            filtering: false,
            confirm_delete: false,
        }
    }
}

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(frame.area());

    render_table(frame, app, chunks[0]);
    render_status_bar(frame, app, chunks[1]);
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("App ID"),
        Cell::from("HDR"),
        Cell::from("VRR"),
        Cell::from("VCache"),
        Cell::from("MangoHud"),
        Cell::from("vkBasalt"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let yn = |opt: Option<bool>| match opt {
        Some(true) => Span::styled("yes", Style::default().fg(Color::Green)),
        Some(false) => Span::styled("no", Style::default().fg(Color::Red)),
        None => Span::styled("-", Style::default().fg(Color::DarkGray)),
    };

    let rows: Vec<Row> = app
        .profiles
        .iter()
        .filter(|(slug, profile)| {
            if app.list_state.filter.is_empty() {
                return true;
            }
            let filter = app.list_state.filter.to_lowercase();
            let name = profile.name.as_deref().unwrap_or(slug).to_lowercase();
            name.contains(&filter) || slug.to_lowercase().contains(&filter)
        })
        .map(|(slug, profile)| {
            let name = profile.name.as_deref().unwrap_or(slug);
            let app_id = profile
                .steam_app_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string());
            let s = profile.settings.as_ref();
            Row::new(vec![
                Cell::from(name.to_string()),
                Cell::from(app_id),
                Cell::from(yn(s.and_then(|s| s.hdr))),
                Cell::from(yn(s.and_then(|s| s.vrr))),
                Cell::from(yn(s.and_then(|s| s.vcache))),
                Cell::from(yn(profile.mangohud.as_ref().and_then(|m| m.enabled))),
                Cell::from(yn(profile.vkbasalt.as_ref().and_then(|v| v.enabled))),
            ])
        })
        .collect();

    let title = if app.list_state.confirm_delete {
        " Profiles — Press 'y' to confirm delete, any other key to cancel "
    } else {
        " Profiles — (n)ew  (e)dit  (d)elete  (/)search  (q)uit "
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(12),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = app.list_state.table_state.clone();
    frame.render_stateful_widget(table, area, &mut state);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let vcache_str = match &app.hardware.vcache {
        Some(vc) => format!("V-Cache: CPUs {} ({}MB L3)", vc.cpus, vc.l3_size_kb / 1024),
        None => "V-Cache: not detected".to_string(),
    };

    let text = if app.list_state.filtering {
        format!("/{} | {vcache_str}", app.list_state.filter)
    } else if let Some(msg) = &app.status_message {
        format!("{msg} | {vcache_str}")
    } else {
        format!("{} profiles | {vcache_str}", app.profiles.len())
    };

    let bar = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(bar, area);
}

pub fn handle_input(app: &mut App, key: KeyEvent) {
    if app.list_state.filtering {
        match key.code {
            KeyCode::Esc => {
                app.list_state.filtering = false;
                app.list_state.filter.clear();
            }
            KeyCode::Enter => {
                app.list_state.filtering = false;
            }
            KeyCode::Backspace => {
                app.list_state.filter.pop();
            }
            KeyCode::Char(c) => {
                app.list_state.filter.push(c);
            }
            _ => {}
        }
        return;
    }

    if app.list_state.confirm_delete {
        app.list_state.confirm_delete = false;
        if key.code == KeyCode::Char('y') {
            if let Some(idx) = app.list_state.table_state.selected()
                && let Some((slug, _)) = app.profiles.get(idx)
            {
                let slug = slug.clone();
                match config::delete_profile(&slug) {
                    Ok(true) => {
                        app.status_message = Some(format!("Deleted '{slug}'"));
                        let _ = app.reload_profiles();
                    }
                    Ok(false) => {
                        app.status_message = Some(format!("'{slug}' not found"));
                    }
                    Err(e) => {
                        app.status_message = Some(format!("Error: {e}"));
                    }
                }
            }
        } else {
            app.status_message = Some("Delete cancelled".to_string());
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => {
            let count = app.profiles.len();
            if count > 0 {
                let i = app.list_state.table_state.selected().unwrap_or(0);
                app.list_state.table_state.select(Some((i + 1) % count));
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let count = app.profiles.len();
            if count > 0 {
                let i = app.list_state.table_state.selected().unwrap_or(0);
                app.list_state
                    .table_state
                    .select(Some(if i == 0 { count - 1 } else { i - 1 }));
            }
        }
        KeyCode::Char('e') | KeyCode::Enter => {
            if let Some(idx) = app.list_state.table_state.selected()
                && idx < app.profiles.len()
            {
                let (slug, profile) = &app.profiles[idx];
                app.edit_state = Some(profile_edit::EditState::from_profile(
                    slug.clone(),
                    profile.clone(),
                    false,
                ));
                app.view = View::Edit(idx);
            }
        }
        KeyCode::Char('n') => {
            app.edit_state = Some(profile_edit::EditState::new_profile());
            app.view = View::New;
        }
        KeyCode::Char('d') => {
            if app.list_state.table_state.selected().is_some() {
                app.list_state.confirm_delete = true;
            }
        }
        KeyCode::Char('/') => {
            app.list_state.filtering = true;
            app.list_state.filter.clear();
        }
        _ => {}
    }
}
