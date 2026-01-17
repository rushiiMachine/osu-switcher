use crate::osu_util::{check_osu_installation, find_osu_installation};
use crate::shortcuts;
use crate::tui::input::InputState;
use color_eyre::eyre::Context;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, HighlightSpacing, List, ListState, Padding, Paragraph, Wrap};
use ratatui::DefaultTerminal;
use std::cmp::PartialEq;
use std::env;
use std::path::{Path, PathBuf};
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
    SelectingOsuDirectory {
        /// The available items for this options list are:
        /// `0`: The found osu! installation directory
        /// `1`: Prompt to enter custom installation directory
        items: ListState,
        default: PathBuf,
    },
    InputtingOsuDirectory { input: InputState, retrying: bool },
    SelectingOsuDomains {
        // FIXME: this should be preserved while adding a new domain
        items: ListState,
    },
    InputtingOsuDomain { input: InputState, retrying: bool },
    Exiting,
}

#[derive(Debug, Default)]
struct App {
    state: AppState,
    osu_dir: Option<PathBuf>,
    osu_domains: Vec<String>,
}

impl App {
    const BANNER: &'static str = "
██████ █████ ██  ██   ██   █████ ██     ██ ██ ██████ █████ ██   ██
██  ██ ██    ██  ██   ██   ██    ██     ██ ██   ██   ██    ██ █ ██
██  ██    ██ ██  ██           ██ ██ ███ ██ ██   ██   ██    ██   ██
██████ █████ ██████   ██   █████  ███ ███  ██   ██   █████ ██   ██

osu!stable server+account switcher to automate re-signing in
https://github.com/rushiiMachine/osu-switcher

Press 'Ctrl+C' to forcefully exit.
";

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
        self.state = match find_osu_installation() {
            Some(path) => AppState::SelectingOsuDirectory {
                items: ListState::default().with_selected(Some(0)),
                default: path,
            },
            None => AppState::InputtingOsuDirectory {
                input: InputState::default(),
                retrying: false,
            }
        };
    }

    /// Handles an input event and returns whether the loop should continue.
    fn update(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle force quit with Ctrl+C
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            exit(1);
        }

        // App has finished and is waiting for any key press to exit
        if self.state == AppState::Exiting && key.kind == KeyEventKind::Press {
            return Ok(false);
        }

        // Enter pressed, triggering some action
        if key.code == KeyCode::Enter && key.kind == KeyEventKind::Press {
            match &mut self.state {
                // User selected osu! install directory options
                AppState::SelectingOsuDirectory { items, default } => {
                    match items.selected() {
                        // Default install path selected
                        Some(0) => {
                            let path = std::mem::replace(default, PathBuf::new());

                            self.osu_dir = Some(path);
                            self.state = AppState::SelectingOsuDomains {
                                items: ListState::default(),
                            };
                        }
                        // Custom install path selected
                        Some(1) => {
                            self.state = AppState::InputtingOsuDirectory {
                                input: InputState::default(),
                                retrying: false,
                            }
                        }
                        _ => {},
                    }
                }

                // User finished entering custom osu! installation path
                AppState::InputtingOsuDirectory { input, retrying } => {
                    let path = PathBuf::from(input.text());

                    if check_osu_installation(&*path) {
                        self.osu_dir = Some(path);
                        self.state = AppState::SelectingOsuDomains {
                            items: ListState::default(),
                        };
                    } else {
                        *retrying = true;
                    }
                }

                // User finished inputting custom osu! server domain
                AppState::InputtingOsuDomain { input, retrying } => {
                    if !input.text().contains(".") && input.text() != "localhost" {
                        *retrying = true;
                    } else {
                        self.osu_domains.push(input.text().to_owned());
                        // FIXME: select newly added domain
                        // self.state = AppState::SelectingOsuDomains {
                        //     items: ListState::default(),
                        // };
                    }
                }

                // User finished selecting osu! server domains
                AppState::SelectingOsuDomains { .. } => {
                    let this_exe = env::current_exe()?;
                    let osu_dir = self.osu_dir.as_deref().unwrap();

                    for server in &self.osu_domains {
                        shortcuts::create_shortcut(&*osu_dir, &*this_exe, &*server);
                    }

                    self.state = AppState::Exiting;
                }

                _ => {}
            }

            // Consume all Enter presses
            return Ok(true);
        }

        // List navigation
        match &mut self.state {
            AppState::SelectingOsuDirectory { items, .. } |
            AppState::SelectingOsuDomains { items } => {
                match key.code {
                    KeyCode::Up => items.select_previous(),
                    KeyCode::Down => items.select_next(),
                    KeyCode::PageUp | KeyCode::Home => items.select_first(),
                    KeyCode::PageDown | KeyCode::End => items.select_last(),
                    _ => {},
                }

                return Ok(true);
            }
            _ => {},
        }

        // Generic text input
        match &mut self.state {
            AppState::InputtingOsuDirectory { input, .. } |
            AppState::InputtingOsuDomain { input, .. } => {
                input.handle_event(key);
                return Ok(true);
            }
            _ => {}
        }

        // No changes, continue
        Ok(true)
    }

    /// Renders the current TUI state to the terminal.
    fn draw(&mut self, frame: &mut Frame) {
        let [banner_area, area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(Self::BANNER.lines().count() as u16),
                Constraint::Min(0),
            ])
            .areas(frame.area());

        let [_, area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(0),
                Constraint::Fill(1),
            ])
            .areas(area);

        let area = area.centered_horizontally(Constraint::Percentage(75));

        Self::draw_banner(frame, banner_area);

        match &mut self.state {
            AppState::Started => { /* No content */ }
            AppState::SelectingOsuDirectory { items, default } =>
                Self::draw_install_dir_picker(frame, area, default, items),
            AppState::InputtingOsuDirectory { .. } => todo!(),
            AppState::SelectingOsuDomains { .. } => todo!(),
            AppState::InputtingOsuDomain { .. } => todo!(),
            AppState::Exiting => Self::draw_exiting(frame, area),
        }
    }

    fn draw_banner(frame: &mut Frame, area: Rect) {
        let banner = Paragraph::new(Self::BANNER)
            .centered()
            .wrap(Wrap { trim: false })
            .gray()
            .dim();

        frame.render_widget(banner, area);
    }

    fn draw_install_dir_picker(
        frame: &mut Frame,
        area: Rect,
        default_dir: &Path,
        list: &mut ListState,
    ) {
        let items = [
            Span::raw(default_dir.to_str()
                .expect("default osu! path contains invalid characters")),
            Span::raw("Custom installation path"),
            // Span::styled("Custom installation path", Style::new().gray().italic().dim())
        ];

        let options = List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Block::new()
                .padding(Padding::uniform(1))
                .title(" osu! install directory ")
                .borders(Borders::ALL)
                .border_style(Style::new().gray().dim()));

        frame.render_stateful_widget(options, area, list);
    }

    fn draw_exiting(frame: &mut Frame, area: Rect) {
        let text = Paragraph::new("Created all shortcuts! Press any key to exit...")
            .green()
            .centered();

        frame.render_widget(text, area);
    }
}
