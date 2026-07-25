use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::config::{self, Profile};
use crate::hardware::{self, HardwareInfo};

mod profile_edit;
mod profile_list;

#[derive(Debug, Clone, PartialEq)]
enum View {
    List,
    Edit(usize),
    New,
}

pub struct App {
    profiles: Vec<(String, Profile)>,
    hardware: HardwareInfo,
    view: View,
    list_state: profile_list::ListState,
    edit_state: Option<profile_edit::EditState>,
    should_quit: bool,
    status_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let profiles = config::list_profiles()?;
        let hardware = hardware::detect_hardware(false).unwrap_or_default();
        Ok(Self {
            list_state: profile_list::ListState::new(profiles.len()),
            profiles,
            hardware,
            view: View::List,
            edit_state: None,
            should_quit: false,
            status_message: None,
        })
    }

    fn reload_profiles(&mut self) -> Result<()> {
        self.profiles = config::list_profiles()?;
        self.list_state = profile_list::ListState::new(self.profiles.len());
        Ok(())
    }

    fn filtered_indices(&self) -> Vec<usize> {
        if self.list_state.filter.is_empty() {
            return (0..self.profiles.len()).collect();
        }
        let filter = self.list_state.filter.to_lowercase();
        self.profiles
            .iter()
            .enumerate()
            .filter(|(_, (slug, profile))| {
                let name = profile.name.as_deref().unwrap_or(slug).to_lowercase();
                name.contains(&filter) || slug.to_lowercase().contains(&filter)
            })
            .map(|(i, _)| i)
            .collect()
    }
}

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
        terminal.draw(|frame| match &app.view {
            View::List => profile_list::render(frame, &app),
            View::Edit(_) | View::New => {
                if let Some(ref edit_state) = app.edit_state {
                    profile_edit::render(frame, edit_state);
                }
            }
        })?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            // Ctrl+C always quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                break;
            }

            match &app.view {
                View::List => profile_list::handle_input(&mut app, key),
                View::Edit(_) | View::New => {
                    profile_edit::handle_input(&mut app, key);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
