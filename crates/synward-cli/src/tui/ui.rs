//! TUI Rendering

use std::io::stdout;
use std::path::PathBuf;

use crate::tui::mcp_screen::PlatformStatus;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};

use crate::tui::app::{App, AppMode, MENU_ITEMS};

const CYAN: Color = Color::Rgb(77, 184, 232);
const YELLOW: Color = Color::Rgb(245, 200, 66);
const DIM: Color = Color::Rgb(122, 174, 200);

pub fn run_tui(project_root: PathBuf) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(project_root);

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Clear status on any key
            if app.status.is_some() && key.code != KeyCode::Enter {
                app.status = None;
            }

            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('c')) |
                (KeyModifiers::NONE, KeyCode::Char('q')) => {
                    if app.mode == AppMode::Menu {
                        app.should_quit = true;
                    } else {
                        app.mode = AppMode::Menu;
                    }
                }
                (KeyModifiers::NONE, KeyCode::Tab) => {
                    app.next_mode();
                }
                (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                    app.prev_mode();
                }
                // Number shortcuts from anywhere
                (KeyModifiers::NONE, KeyCode::Char('1')) => app.mode = AppMode::Dashboard,
                (KeyModifiers::NONE, KeyCode::Char('2')) => app.mode = AppMode::Validate,
                (KeyModifiers::NONE, KeyCode::Char('3')) => app.mode = AppMode::Config,
                (KeyModifiers::NONE, KeyCode::Char('4')) => app.mode = AppMode::Memory,
                (KeyModifiers::NONE, KeyCode::Char('5')) => app.mode = AppMode::McpSetup,
                (KeyModifiers::NONE, KeyCode::Char('?')) => app.mode = AppMode::Help,
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    handle_enter(app)?;
                }
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    if app.mode != AppMode::Menu {
                        app.mode = AppMode::Menu;
                    } else {
                        app.should_quit = true;
                    }
                }
                (KeyModifiers::NONE, KeyCode::Up | KeyCode::Char('k')) => {
                    handle_up(app);
                }
                (KeyModifiers::NONE, KeyCode::Down | KeyCode::Char('j')) => {
                    handle_down(app);
                }
                // Dubbioso preset cycling in Config screen
                (KeyModifiers::NONE, KeyCode::Left) if app.mode == AppMode::Config && app.config_screen.selected == 0 => {
                    app.config_screen.prev_dubbioso_preset();
                    if app.config_screen.config.is_some() {
                        app.status = Some(format!("Preset: {}", app.config_screen.dubbioso_preset_name()));
                    }
                }
                (KeyModifiers::NONE, KeyCode::Right) if app.mode == AppMode::Config && app.config_screen.selected == 0 => {
                    app.config_screen.next_dubbioso_preset();
                    if app.config_screen.config.is_some() {
                        app.status = Some(format!("Preset: {}", app.config_screen.dubbioso_preset_name()));
                    }
                }
                // MCP: 'w' to write config
                (KeyModifiers::NONE, KeyCode::Char('w')) if app.mode == AppMode::McpSetup => {
                    match app.mcp_screen.write_config() {
                        Ok(msg) => app.status = Some(msg),
                        Err(e) => app.status = Some(format!("Error: {}", e)),
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_enter(app: &mut App) -> anyhow::Result<()> {
    match app.mode {
        AppMode::Menu => {
            match app.menu_selected {
                0 => app.mode = AppMode::Dashboard,
                1 => app.mode = AppMode::Validate,
                2 => app.mode = AppMode::Config,
                3 => app.mode = AppMode::Memory,
                4 => app.mode = AppMode::McpSetup,
                5 => app.mode = AppMode::Help,
                _ => {}
            }
        }
        AppMode::Config => {
            if app.config_screen.config.is_none() {
                app.config_screen.load_or_create()?;
                app.config_screen.apply_dubbioso_preset(); // Apply default preset
                app.status = Some("Default config created (preset: Balanced)".to_string());
            } else {
                app.config_screen.save()?;
                app.status = Some(format!("Saved! Preset: {}", app.config_screen.dubbioso_preset_name()));
            }
        }
        AppMode::McpSetup => {
            match app.mcp_screen.write_config() {
                Ok(msg) => app.status = Some(msg),
                Err(e) => app.status = Some(format!("Error: {}", e)),
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_up(app: &mut App) {
    match app.mode {
        AppMode::Menu => {
            if app.menu_selected > 0 {
                app.menu_selected -= 1;
            }
        }
        AppMode::Config => {
            if app.config_screen.selected > 0 {
                app.config_screen.selected -= 1;
            }
        }
        AppMode::Memory => {
            if app.memory_screen.selected > 0 {
                app.memory_screen.selected -= 1;
            }
        }
        AppMode::McpSetup => {
            if app.mcp_screen.selected > 0 {
                app.mcp_screen.selected -= 1;
            }
        }
        _ => {}
    }
}

fn handle_down(app: &mut App) {
    match app.mode {
        AppMode::Menu => {
            app.menu_selected = (app.menu_selected + 1).min(MENU_ITEMS.len() - 1);
        }
        AppMode::Config => {
            let max = app.config_screen.fields().len().saturating_sub(1);
            app.config_screen.selected = (app.config_screen.selected + 1).min(max);
        }
        AppMode::Memory => {
            let max = app.memory_screen.entries.len().saturating_sub(1);
            app.memory_screen.selected = (app.memory_screen.selected + 1).min(max);
        }
        AppMode::McpSetup => {
            let max = app.mcp_screen.platforms.len().saturating_sub(1);
            app.mcp_screen.selected = (app.mcp_screen.selected + 1).min(max);
        }
        _ => {}
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);

    match app.mode {
        AppMode::Menu => render_menu(f, app, chunks[1]),
        AppMode::Dashboard => render_dashboard(f, app, chunks[1]),
        AppMode::Validate => render_validate(f, app, chunks[1]),
        AppMode::Config => render_config(f, app, chunks[1]),
        AppMode::Memory => render_memory(f, app, chunks[1]),
        AppMode::McpSetup => render_mcp(f, app, chunks[1]),
        AppMode::Help => render_help(f, chunks[1]),
    }

    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let title = Line::from(vec![
        Span::styled(" SYNWARD ", Style::default().fg(CYAN).add_modifier(Modifier::BOLD)),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", app.mode), Style::default().fg(YELLOW)),
    ]);
    let header = Paragraph::new(title)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(header, area);
}

fn render_footer(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let status_line = if let Some(ref status) = app.status {
        Line::from(Span::styled(format!(" {}", status), Style::default().fg(Color::Green)))
    } else {
        Line::from(vec![
            Span::styled(" [1]", Style::default().fg(YELLOW)),
            Span::styled("Dash ", Style::default().fg(DIM)),
            Span::styled("[2]", Style::default().fg(YELLOW)),
            Span::styled("Valid ", Style::default().fg(DIM)),
            Span::styled("[3]", Style::default().fg(YELLOW)),
            Span::styled("Conf ", Style::default().fg(DIM)),
            Span::styled("[4]", Style::default().fg(YELLOW)),
            Span::styled("Mem ", Style::default().fg(DIM)),
            Span::styled("[5]", Style::default().fg(YELLOW)),
            Span::styled("MCP ", Style::default().fg(DIM)),
            Span::styled("[?]", Style::default().fg(YELLOW)),
            Span::styled("Help ", Style::default().fg(DIM)),
            Span::styled("[q]", Style::default().fg(YELLOW)),
            Span::styled("Quit", Style::default().fg(DIM)),
        ])
    };
    let footer = Paragraph::new(status_line)
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(footer, area);
}

fn render_menu(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // Left: menu items
    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let number = if i < 5 { format!("{}", i + 1) } else { "?".to_string() };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", number), Style::default().fg(YELLOW)),
                Span::styled(*name, Style::default().fg(Color::White)),
                Span::styled(format!("  {}", desc), Style::default().fg(DIM)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(" Main Menu ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(
            Style::default()
                .fg(YELLOW)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("→ ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.menu_selected));
    f.render_stateful_widget(list, chunks[0], &mut state);

    // Right: quick info
    let info_lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Welcome to Synward", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  Universal AI Code Validator", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  Quick Start:", Style::default().fg(YELLOW))),
        Line::from(Span::styled("  1. Configure your project [3]", Style::default().fg(DIM))),
        Line::from(Span::styled("  2. Set up MCP for your editor [5]", Style::default().fg(DIM))),
        Line::from(Span::styled("  3. Run validation [2]", Style::default().fg(DIM))),
        Line::from(""),
        Line::from(Span::styled("  Navigation:", Style::default().fg(YELLOW))),
        Line::from(Span::styled("  ↑↓/jk  Navigate", Style::default().fg(DIM))),
        Line::from(Span::styled("  Enter   Select", Style::default().fg(DIM))),
        Line::from(Span::styled("  Tab     Next screen", Style::default().fg(DIM))),
        Line::from(Span::styled("  Esc     Back to menu", Style::default().fg(DIM))),
    ];

    let info = Paragraph::new(info_lines)
        .block(Block::default()
            .title(" Quick Start ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(info, chunks[1]);
}

fn render_dashboard(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Project info
            Constraint::Min(0),     // Features
        ])
        .split(area);

    // Project info
    let config_status = if app.config_screen.config.is_some() { "Loaded" } else { "Not found" };
    let config_color = if app.config_screen.config.is_some() { Color::Green } else { Color::Red };
    let memory_count = app.memory_screen.entries.len();

    let project_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Project: ", Style::default().fg(DIM)),
            Span::styled(
                app.config_screen.project_root.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| app.config_screen.project_root.display().to_string()),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            ),
        ]),
        Line::from(vec![
            Span::styled("  Config:  ", Style::default().fg(DIM)),
            Span::styled(config_status, Style::default().fg(config_color)),
            Span::styled("  (.synward.toml)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  Memory:  ", Style::default().fg(DIM)),
            Span::styled(format!("{} entries", memory_count), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Path:    ", Style::default().fg(DIM)),
            Span::styled(app.config_screen.project_root.display().to_string(), Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let project = Paragraph::new(project_lines)
        .block(Block::default()
            .title(" Project Status ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(project, chunks[0]);

    // Features overview
    let features_lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Available Features", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  validate <file>    ", Style::default().fg(YELLOW)),
            Span::styled("Validate code with contracts and rules", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  certify <file>     ", Style::default().fg(YELLOW)),
            Span::styled("Sign validated code with Ed25519", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  analyze <file>     ", Style::default().fg(YELLOW)),
            Span::styled("AST analysis and metrics", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  discover <path>    ", Style::default().fg(YELLOW)),
            Span::styled("Discover patterns and anomalies", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  drift [--commits]  ", Style::default().fg(YELLOW)),
            Span::styled("Detect code quality drift over time", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  memory recall      ", Style::default().fg(YELLOW)),
            Span::styled("Query semantic memory (why-exists, who-calls)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  config --show      ", Style::default().fg(YELLOW)),
            Span::styled("Show project configuration", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  contracts update    ", Style::default().fg(YELLOW)),
            Span::styled("Update validation contracts from registry", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Languages: Rust, C++, Python, JS, TS, Go, Java, Lua, Lex, C, GLSL, CSS, HTML, JSON, YAML", Style::default().fg(DIM))),
        Line::from(Span::styled("  Output:    --format json  for machine-readable output", Style::default().fg(DIM))),
    ];

    let features = Paragraph::new(features_lines)
        .block(Block::default()
            .title(" CLI Commands ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(features, chunks[1]);
}

fn render_validate(f: &mut ratatui::Frame, _app: &App, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Validation", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  Run from CLI:", Style::default().fg(YELLOW))),
        Line::from(""),
        Line::from(Span::styled("    synward validate <file>                       # Single file", Style::default().fg(Color::White))),
        Line::from(Span::styled("    synward validate src/                         # Directory", Style::default().fg(Color::White))),
        Line::from(Span::styled("    synward validate src/main.rs --lang rust      # Explicit language", Style::default().fg(Color::White))),
        Line::from(Span::styled("    synward validate src/ --format json           # JSON output", Style::default().fg(Color::White))),
        Line::from(Span::styled("    synward validate src/ --severity error        # Errors only", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  Validation Layers:", Style::default().fg(YELLOW))),
        Line::from(""),
        Line::from(vec![
            Span::styled("    1. ", Style::default().fg(YELLOW)),
            Span::styled("Syntax      ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Parse errors, malformed code", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("    2. ", Style::default().fg(YELLOW)),
            Span::styled("Semantic    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Type mismatches, scope issues", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("    3. ", Style::default().fg(YELLOW)),
            Span::styled("Logic       ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Dead code, unreachable branches", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("    4. ", Style::default().fg(YELLOW)),
            Span::styled("Security    ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Hardcoded secrets, injection risks", Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("    5. ", Style::default().fg(YELLOW)),
            Span::styled("Style       ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("Naming, complexity, conventions", Style::default().fg(DIM)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Accept violations:", Style::default().fg(YELLOW))),
        Line::from(Span::styled("    synward validate file.rs --accept SYN001,SEC002 --reason \"intended\"", Style::default().fg(Color::White))),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title(" Validate ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_config(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let config = &app.config_screen.config;

    let mut lines: Vec<Line> = vec![
        Line::from(""),
    ];

    // Dubbioso Preset Selector (always first, highlightable)
    let preset_name = app.config_screen.dubbioso_preset_name();
    let preset_desc = app.config_screen.dubbioso_preset_desc();
    let is_selected = app.config_screen.selected == 0;

    lines.push(Line::from(vec![
        Span::styled(if is_selected { "→ " } else { "  " }, Style::default().fg(if is_selected { YELLOW } else { Color::DarkGray })),
        Span::styled("Dubbioso Mode:  ", Style::default().fg(DIM)),
        Span::styled("← ", Style::default().fg(if is_selected { Color::White } else { Color::DarkGray })),
        Span::styled(format!(" {} ", preset_name), Style::default().fg(if is_selected { YELLOW } else { Color::White }).add_modifier(Modifier::BOLD)),
        Span::styled(" →", Style::default().fg(if is_selected { Color::White } else { Color::DarkGray })),
    ]));
    lines.push(Line::from(vec![
        Span::styled("                 ", Style::default().fg(DIM)),
        Span::styled(preset_desc, Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    if let Some(cfg) = config {
        lines.push(Line::from(Span::styled("  Configuration (.synward.toml)", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  Version:      ", Style::default().fg(DIM)),
            Span::styled(cfg.version.as_deref().unwrap_or("1.0"), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Thresholds:   ", Style::default().fg(DIM)),
            Span::styled(format!("{} entries", cfg.thresholds.len()), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Whitelist:    ", Style::default().fg(DIM)),
            Span::styled(format!("{} entries", cfg.whitelist.entries.len()), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Custom Rules: ", Style::default().fg(DIM)),
            Span::styled(format!("{} entries", cfg.rules.custom.len()), Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  [Enter] Save  |  [←/→] Change preset  |  Edit .synward.toml for full control", Style::default().fg(YELLOW))));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  No .synward.toml found", Style::default().fg(Color::Red))));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  [Enter] Create default template", Style::default().fg(YELLOW))));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title(" Configuration ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(paragraph, area);
}

fn render_memory(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let entries = &app.memory_screen.entries;

    let lines: Vec<Line> = if entries.is_empty() {
        vec![
            Line::from(""),
            Line::from(Span::styled("  No memory entries yet", Style::default().fg(DIM))),
            Line::from(""),
            Line::from(Span::styled("  Memory entries are created when you:", Style::default().fg(YELLOW))),
            Line::from(Span::styled("    - Accept violations (--accept)", Style::default().fg(Color::White))),
            Line::from(Span::styled("    - Adjust thresholds in .synward.toml", Style::default().fg(Color::White))),
            Line::from(Span::styled("    - Run drift analysis", Style::default().fg(Color::White))),
            Line::from(Span::styled("    - Discover patterns", Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled("  CLI commands:", Style::default().fg(YELLOW))),
            Line::from(Span::styled("    synward memory list                    List all entries", Style::default().fg(DIM))),
            Line::from(Span::styled("    synward memory recall why-exists <fn>  Why does this code exist?", Style::default().fg(DIM))),
            Line::from(Span::styled("    synward memory recall who-calls <fn>   Who calls this function?", Style::default().fg(DIM))),
            Line::from(Span::styled("    synward memory recall search <query>   Semantic search", Style::default().fg(DIM))),
        ]
    } else {
        let mut l = vec![
            Line::from(""),
            Line::from(Span::styled("  Memory Entries", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))),
            Line::from(""),
        ];
        for (i, entry) in entries.iter().enumerate() {
            let style = if i == app.memory_screen.selected {
                Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if i == app.memory_screen.selected { "→ " } else { "  " };
            l.push(Line::from(Span::styled(
                format!("{}[{}] {} — {}", prefix, entry.kind_str(), entry.id, entry.title),
                style,
            )));
        }
        l
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title(" Memory Browser ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(paragraph, area);
}

fn render_mcp(f: &mut ratatui::Frame, app: &App, area: Rect) {
    if app.mcp_screen.platforms.is_empty() {
        let lines = vec![
            Line::from(""),
            Line::from(Span::styled("  No MCP-compatible platforms detected", Style::default().fg(Color::Red))),
        ];
        let paragraph = Paragraph::new(lines)
            .block(Block::default()
                .title(" MCP Setup ")
                .title_style(Style::default().fg(CYAN))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(paragraph, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ])
        .split(area);

    // Left: platform list with status indicators
    let detected_count = app.mcp_screen.platforms.iter().filter(|p| p.status != PlatformStatus::NotFound).count();
    let configured_count = app.mcp_screen.platforms.iter().filter(|p| p.status == PlatformStatus::Configured).count();

    let items: Vec<ListItem> = app.mcp_screen.platforms
        .iter()
        .map(|p| {
            let (icon, color) = match p.status {
                PlatformStatus::Configured => ("✓", Color::Green),
                PlatformStatus::Detected => ("●", YELLOW),
                PlatformStatus::NotFound => ("○", Color::DarkGray),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                Span::styled(p.name, Style::default().fg(if p.status == PlatformStatus::NotFound { Color::DarkGray } else { Color::White })),
                Span::styled(format!("  {}", p.status.label()), Style::default().fg(color)),
            ]))
        })
        .collect();

    let title = format!(" Platforms ({} detected, {} configured) ", detected_count, configured_count);
    let list = List::new(items)
        .block(Block::default()
            .title(title)
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)))
        .highlight_style(Style::default().fg(YELLOW).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");

    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.mcp_screen.selected));
    f.render_stateful_widget(list, chunks[0], &mut state);

    // Right: config preview for selected platform
    let platform = &app.mcp_screen.platforms[app.mcp_screen.selected];
    let config_preview = app.mcp_screen.generate_config().unwrap_or_else(|e| format!("Error: {}", e));

    let status_line = match platform.status {
        PlatformStatus::Configured => Line::from(Span::styled("  Status: Synward MCP already configured", Style::default().fg(Color::Green))),
        PlatformStatus::Detected => Line::from(Span::styled("  Status: Platform found, Synward not configured", Style::default().fg(YELLOW))),
        PlatformStatus::NotFound => Line::from(Span::styled("  Status: Platform not installed", Style::default().fg(Color::DarkGray))),
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {}", platform.description), Style::default().fg(DIM))),
        status_line,
        Line::from(""),
        Line::from(Span::styled("  Config path:", Style::default().fg(YELLOW))),
        Line::from(Span::styled(format!("  {}", platform.config_path.display()), Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  Generated config:", Style::default().fg(YELLOW))),
        Line::from(""),
    ];

    for line in config_preview.lines() {
        lines.push(Line::from(Span::styled(format!("  {}", line), Style::default().fg(Color::White))));
    }

    lines.push(Line::from(""));
    let action_hint = if platform.status == PlatformStatus::Configured {
        "  [Enter/w] Overwrite config  |  [↑↓] Switch platform"
    } else {
        "  [Enter/w] Write config  |  [↑↓] Switch platform"
    };
    lines.push(Line::from(Span::styled(action_hint, Style::default().fg(YELLOW))));

    let preview = Paragraph::new(lines)
        .block(Block::default()
            .title(" MCP Configuration ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)))
        .wrap(Wrap { trim: false });
    f.render_widget(preview, chunks[1]);
}

fn render_help(f: &mut ratatui::Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Synward TUI — Help", Style::default().fg(CYAN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  Navigation", Style::default().fg(YELLOW))),
        Line::from(Span::styled("    ↑↓ / j/k     Navigate items", Style::default().fg(Color::White))),
        Line::from(Span::styled("    Enter         Select / Confirm", Style::default().fg(Color::White))),
        Line::from(Span::styled("    Tab           Next screen", Style::default().fg(Color::White))),
        Line::from(Span::styled("    Shift+Tab     Previous screen", Style::default().fg(Color::White))),
        Line::from(Span::styled("    Esc           Back to menu", Style::default().fg(Color::White))),
        Line::from(Span::styled("    q / Ctrl+C    Quit", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  Screen Shortcuts", Style::default().fg(YELLOW))),
        Line::from(Span::styled("    1  Dashboard      Project overview", Style::default().fg(Color::White))),
        Line::from(Span::styled("    2  Validate       Validation guide", Style::default().fg(Color::White))),
        Line::from(Span::styled("    3  Config         Edit .synward.toml", Style::default().fg(Color::White))),
        Line::from(Span::styled("    4  Memory         Browse learned data", Style::default().fg(Color::White))),
        Line::from(Span::styled("    5  MCP Setup      Configure MCP for platform", Style::default().fg(Color::White))),
        Line::from(Span::styled("    ?  Help           This screen", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  MCP Setup Screen", Style::default().fg(YELLOW))),
        Line::from(Span::styled("    w / Enter     Write config to selected platform", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("  What is Synward?", Style::default().fg(YELLOW))),
        Line::from(Span::styled("    Synward validates AI-generated code through formal rules,", Style::default().fg(DIM))),
        Line::from(Span::styled("    contracts, and learned patterns. It works as:", Style::default().fg(DIM))),
        Line::from(Span::styled("    - CLI tool (synward validate, certify, analyze)", Style::default().fg(DIM))),
        Line::from(Span::styled("    - MCP server for AI agents (Droid, Claude, etc.)", Style::default().fg(DIM))),
        Line::from(Span::styled("    - VS Code Extension (planned)", Style::default().fg(DIM))),
    ];

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .title(" Help ")
            .title_style(Style::default().fg(CYAN))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(paragraph, area);
}
