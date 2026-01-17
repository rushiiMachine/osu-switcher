use crate::osu_util::{check_osu_installation, find_osu_installation, flatten_osu_installation};
use crate::shortcuts;
use crate::tui::input::InputState;
use color_eyre::eyre::Context;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListState, Padding, Paragraph, Wrap,
};
use std::cmp::PartialEq;
use std::path::{Path, PathBuf};

mod input;

pub fn start_tui() {
    let mut app = App::default();

    ratatui::run(|terminal| app.run(terminal))
        .context("app loop failed")
        .unwrap();
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
    InputtingOsuDirectory {
        input: InputState,
        retrying: bool,
    },
    SelectingOsuDomains {
        items: ListState,
    },
    InputtingOsuDomain {
        input: InputState,
        retrying: bool,
    },
    Exiting,
}

#[derive(Debug)]
struct ServerState {
    domain: String,
    enabled: bool,
}

#[derive(Debug, Default)]
struct App {
    state: AppState,
    osu_dir: Option<PathBuf>,
    osu_servers: Vec<ServerState>,
}

impl App {
    const BANNER: &'static str = "
██████ █████ ██  ██   ██   █████ ██     ██ ██ ██████ █████ ██   ██
██  ██ ██    ██  ██   ██   ██    ██     ██ ██   ██   ██    ██   ██
██  ██    ██ ██  ██           ██ ██ ███ ██ ██   ██   ██    ██ █ ██
██████ █████ ██████   ██   █████  ███ ███  ██   ██   █████ ██   ██

osu!stable server+account switcher to automate re-signing in
https://github.com/rushiiMachine/osu-switcher

Press 'Ctrl+C' to forcefully exit.
";

    /// Main loop that triggers rendering and processing input.
    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        <B as Backend>::Error: Send + Sync + 'static,
    {
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
            },
        };

        let mut known_servers = shortcuts::ICONS.keys().collect::<Vec<_>>();
        known_servers.sort_unstable();

        for domain in known_servers {
            self.osu_servers.push(ServerState {
                domain: (*domain).to_owned(),
                enabled: false,
            })
        }
    }

    /// Handles an input event and returns whether the loop should continue.
    fn update(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle force quit with Ctrl+C
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(false);
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
                        _ => {}
                    }
                }

                // User finished entering custom osu! installation path
                AppState::InputtingOsuDirectory { input, retrying } => {
                    let path = PathBuf::from(input.text());
                    let osu_dir = flatten_osu_installation(&*path);

                    if check_osu_installation(&*osu_dir) {
                        self.osu_dir = Some(osu_dir.into_owned());
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
                        self.osu_servers.push(ServerState {
                            domain: input.text().to_owned(),
                            enabled: true,
                        });
                        self.state = AppState::SelectingOsuDomains {
                            items: ListState::default()
                                .with_selected(Some(self.osu_servers.len() - 1)),
                        };
                    }
                }

                // User finished selecting osu! server domains
                AppState::SelectingOsuDomains { items } => {
                    // Check if custom server option was selected
                    if let Some(idx) = items.selected()
                        && idx >= self.osu_servers.len()
                    {
                        self.state = AppState::InputtingOsuDomain {
                            input: InputState::default(),
                            retrying: false,
                        }
                    } else {
                        let osu_dir = self.osu_dir.as_deref().unwrap();
                        let servers = self.osu_servers.iter()
                            .filter(|server| server.enabled)
                            .map(|server| &*server.domain);

                        shortcuts::install(osu_dir, servers)?;
                        self.state = AppState::Exiting;
                    }
                }

                _ => {}
            }

            // Consume all Enter presses
            return Ok(true);
        }

        // Selecting osu! domains from list
        if key.code == KeyCode::Char(' ') && key.kind == KeyEventKind::Press {
            if let AppState::SelectingOsuDomains { items } = &self.state {
                let selected_idx = items.selected();

                if let Some(idx) = selected_idx
                    && let Some(server) = self.osu_servers.get_mut(idx)
                {
                    server.enabled = !server.enabled;
                }
            }
        }

        // List navigation
        if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            if let AppState::SelectingOsuDirectory { items, .. }
            | AppState::SelectingOsuDomains { items } = &mut self.state
            {
                match key.code {
                    KeyCode::Up => items.select_previous(),
                    KeyCode::Down => items.select_next(),
                    KeyCode::PageUp | KeyCode::Home => items.select_first(),
                    KeyCode::PageDown | KeyCode::End => items.select_last(),
                    _ => {}
                }

                return Ok(true);
            }
        }

        // Generic text input
        match &mut self.state {
            AppState::InputtingOsuDirectory { input, .. }
            | AppState::InputtingOsuDomain { input, .. } => {
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
                Constraint::Fill(4),
                Constraint::Fill(1),
            ])
            .areas(area);
        let area = area.centered_horizontally(Constraint::Percentage(75));

        Self::draw_banner(frame, banner_area);

        match &mut self.state {
            AppState::Started => { /* No content */ }
            AppState::SelectingOsuDirectory { items, default } => {
                Self::draw_install_dir_picker(frame, area, default, items);
            }
            AppState::InputtingOsuDirectory { input, retrying } => Self::draw_text_box(
                frame,
                area,
                input,
                " Enter osu! installation directory (eg. 'D:\\osu!') ",
                if *retrying {
                    Some("Invalid osu! installation! Please try again.")
                } else {
                    None
                },
            ),
            AppState::SelectingOsuDomains { items } => {
                Self::draw_domains_picker(frame, area, &*self.osu_servers, items);
            }
            AppState::InputtingOsuDomain { input, retrying } => Self::draw_text_box(
                frame,
                area,
                input,
                " Enter new osu! private server domain (eg. 'akatsuki.pw') ",
                if *retrying {
                    Some("Invalid domain! Please try again.")
                } else {
                    None
                },
            ),
            AppState::Exiting => Self::draw_exiting(frame, area),
        };
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
        let default_dir_str = default_dir
            .to_str()
            .expect("default osu! path contains invalid characters");

        let items = [
            Span::raw(default_dir_str).green().bold(),
            Span::raw("Enter a custom osu! installation path...")
                .italic()
                .gray(),
        ];

        let options = List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(
                Self::block_border()
                    .padding(Padding::vertical(1))
                    .title(" osu! install directory "),
            );

        let area = area
            .resize(Size::new(area.width, 6))
            .centered_vertically(Constraint::Length(6));

        frame.render_stateful_widget(options, area, list);
    }

    fn draw_domains_picker(
        frame: &mut Frame,
        area: Rect,
        osu_domains: &[ServerState],
        list: &mut ListState,
    ) {
        let items = osu_domains.iter().map(|server| {
            let style = if server.enabled {
                Style::default().bold().underlined().green()
            } else {
                Style::default().gray()
            };

            Span::styled(&*server.domain, style)
        });

        let items = items.chain([Span::raw("Enter a custom osu! private server...")
            .italic()
            .gray()]);

        let options = List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always)
            .block(Self::block_border().padding(Padding::vertical(1)).title(
                " osu! private server domains (Press 'Space' to select, and 'Enter' to continue) ",
            ));

        frame.render_stateful_widget(options, area, list);
    }

    fn draw_text_box(
        frame: &mut Frame,
        area: Rect,
        input: &InputState,
        title: &str,
        error: Option<&str>,
    ) {
        let all_area = area
            .resize(Size::new(area.width, 4))
            .centered_vertically(Constraint::Length(4));
        let [text_area, notice_area] = all_area.layout(&Layout::vertical([
            Constraint::Min(0),
            Constraint::Length(1),
        ]));

        let block = Self::block_border()
            .padding(Padding::left(1))
            .border_style(if error.is_some() {
                Style::new().red()
            } else {
                Style::new().gray().dim()
            })
            .title(title);

        let input_text = Paragraph::new(input.text()).block(block);
        frame.render_widget(input_text, text_area);

        if let Some(error) = error {
            let notice_text = Paragraph::new(error).red();
            frame.render_widget(notice_text, notice_area);
        }

        frame.set_cursor_position(Position::new(
            // Draw the cursor at the current position in the input field.
            // Offset by 2 to account for border & padding
            area.x + input.position() as u16 + 2,
            // Move one line down, from the border to the input line
            area.y + 1,
        ))
    }

    fn block_border() -> Block<'static> {
        Block::new()
            .borders(Borders::ALL)
            .border_style(Style::new().gray().dim())
    }

    fn draw_exiting(frame: &mut Frame, area: Rect) {
        let text = Paragraph::new("Created all shortcuts! Press any key to exit...")
            .green()
            .centered();

        frame.render_widget(text, area);
    }
}
