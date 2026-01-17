use crate::osu_util::{check_osu_installation, get_osu_installation};
use crate::shortcuts;
use crate::tui::input::InputState;
use color_eyre::eyre::Context;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::DefaultTerminal;
use std::cmp::PartialEq;
use std::env;
use std::path::PathBuf;
use std::process::exit;

mod input;

pub fn start_config() {
    let mut terminal: DefaultTerminal = ratatui::init();
    let mut app = App::default();

    app.run(&mut terminal).context("app loop failed").unwrap();
}

#[derive(Debug, Default, Eq, PartialEq)]
enum AppState {
    #[default]
    Started,
    AddingOsuDirectory { input: InputState, retrying: bool },
    SelectingOsuDomains,
    AddingOsuDomain { input: InputState, retrying: bool },
    Exiting,
}

#[derive(Debug, Default)]
struct App {
    state: AppState,
    osu_dir: Option<PathBuf>,
    osu_domains: Vec<String>,
}

impl App {
    /// Main loop that triggers rendering and processing input.
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.init();

        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if !self.update(key)? {
                    break Ok(());
                }
            }
        }
    }

    /// Initializes the app and performs pre-rendering checks to set up state.
    fn init(&mut self) {
        if let Some(path) = get_osu_installation() {
            self.osu_dir = Some(path);
            self.state = AppState::SelectingOsuDomains;
        } else {
            self.state = AppState::AddingOsuDirectory { input: InputState::default(), retrying: false };
        }
    }

    /// Handles an input event and returns whether the loop should continue.
    fn update(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle force quit with Ctrl+C
        if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
            exit(1);
        }

        // App has finished and is waiting for any key press to exit
        if self.state == AppState::Exiting && key.kind == KeyEventKind::Press {
            return Ok(false);
        }

        // Enter pressed while user is inputting custom osu! installation
        if let AppState::AddingOsuDirectory { input, retrying } = &mut self.state {
            if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
                let path = PathBuf::from(input.text());

                if check_osu_installation(&*path) {
                    self.state = AppState::SelectingOsuDomains;
                    self.osu_dir = Some(path);
                } else {
                    *retrying = true;
                }

                return Ok(true);
            }
        }

        // Enter pressed while user was inputting custom osu! server domain
        if let AppState::AddingOsuDomain { input, retrying } = &mut self.state {
            if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
                if !input.text().contains(".") && input.text() != "localhost" {
                    *retrying = true;
                } else {
                    self.osu_domains.push(input.text().to_owned());
                    self.state = AppState::SelectingOsuDomains;
                }

                return Ok(true);
            }
        }

        // Enter pressed while selecting osu! server domains (finished)
        if self.state == AppState::SelectingOsuDomains &&
            key.code == KeyCode::Enter && key.kind == KeyEventKind::Press
        {
            let this_exe = env::current_exe()?;
            let osu_dir = self.osu_dir.as_deref().unwrap();

            for server in &self.osu_domains {
                shortcuts::create_shortcut(&*osu_dir, &*this_exe, &*server);
            }

            self.state = AppState::Exiting;

            return Ok(true);
        }

        // Generic text input handling
        match &mut self.state {
            AppState::AddingOsuDirectory { input, .. } |
            AppState::AddingOsuDomain { input, .. } => {
                input.handle_event(key);
                return Ok(true);
            }
            _ => {}
        }

        // No changes, continue
        Ok(true)
    }

    /// Renders the current TUI state to the terminal.
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::vertical([Constraint::Length(100), Constraint::Min(1)]);
        let [banner_area, working_area] = layout.areas(frame.area());

        Self::draw_banner(frame, banner_area);

        let greeting = Paragraph::new("Hello World! (press any key to quit)")
            .white()
            .bold();
        frame.render_widget(greeting, working_area);
    }

    fn draw_banner(frame: &mut Frame, area: Rect) {
        const BANNER: &'static str = "
██████ █████ ██  ██   ██   █████ ██     ██ ██ ██████ █████ ██   ██
██  ██ ██    ██  ██   ██   ██    ██     ██ ██   ██   ██    ██ █ ██
██  ██    ██ ██  ██           ██ ██ ███ ██ ██   ██   ██    ██   ██
██████ █████ ██████   ██   █████  ███ ███  ██   ██   █████ ██   ██

osu!stable server+account switcher to automate re-signing in
https://github.com/rushiiMachine/osu-switcher

Press 'Ctrl+C' to forcefully exit.
";

        let banner = Paragraph::new(BANNER)
            .centered()
            .wrap(Wrap { trim: false })
            .gray()
            .dim();

        frame.render_widget(banner, area);
    }
}
