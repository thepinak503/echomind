use crate::api::{ApiClient, ChatRequest, Message, Provider};
use crate::cli::Args;
use crate::config::Config;
use crate::error::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Position},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use tokio::sync::mpsc;
use tokio::task;

#[derive(Debug, Clone)]
enum AppState {
    Input,
    Processing,
    Response,
}

#[derive(Debug)]
struct App {
    state: AppState,
    input: String,
    response: String,
    provider: Provider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
    stream: bool,
    history: Vec<String>,
    history_index: Option<usize>,
    config: Config,
    args: Args,
}

impl App {
    fn new(config: Config, args: Args) -> Self {
        let provider = Provider::from_string(args.provider.as_ref().unwrap_or(&config.api.provider)).unwrap_or(Provider::Chat);
        let model = args.model.as_ref().unwrap_or(&config.api.model).clone();
        let temperature = args.temperature.unwrap_or(config.defaults.temperature);
        let max_tokens = args.max_tokens.or(config.defaults.max_tokens);
        let top_p = args.top_p.or(config.defaults.top_p);
        let top_k = args.top_k.or(config.defaults.top_k);
        let stream = args.stream;

        Self {
            state: AppState::Input,
            input: String::new(),
            response: String::new(),
            provider,
            model,
            temperature,
            max_tokens,
            top_p,
            top_k,
            stream,
            history: Vec::new(),
            history_index: None,
            config,
            args,
        }
    }

    fn next(&mut self) {
        self.state = match self.state {
            AppState::Input => AppState::Processing,
            AppState::Processing => AppState::Response,
            AppState::Response => AppState::Input,
        };
    }
}

pub async fn run_tui(args: Args, config: Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(config, args);
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let AppState::Processing = app.state {
            // Start processing in background
            let input = app.input.clone();
            let provider = app.provider.clone();
            let model = app.model.clone();
            let temperature = app.temperature;
            let max_tokens = app.max_tokens;
            let top_p = app.top_p;
            let top_k = app.top_k;
            let stream = app.stream;
            let config = app.config.clone();
            let args = app.args.clone();
            let tx_process = tx.clone();
            let tx_error = tx.clone();

            task::spawn(async move {
                if let Err(e) = process_query(input, provider, model, temperature, max_tokens, top_p, top_k, stream, config, args, tx_process).await {
                    let _ = tx_error.send(format!("Error: {:?}", e));
                }
            });

            app.next();
        }

        if let Ok(event) = event::read() {
            match event {
                Event::Key(key) => {
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('t') => {
                                // Cycle temperature
                                app.temperature = match app.temperature {
                                    0.1 => 0.5,
                                    0.5 => 1.0,
                                    _ => 0.1,
                                };
                            }
                            KeyCode::Char('s') => {
                                // Toggle stream
                                app.stream = !app.stream;
                            }
                            KeyCode::Char('h') => {
                                // Clear history
                                app.history.clear();
                                app.history_index = None;
                            }
                            KeyCode::Char('r') => {
                                // Clear response
                                app.response.clear();
                                app.state = AppState::Input;
                            }
                            KeyCode::Char('q') => {
                                // Quit
                                return Ok(());
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Enter => {
                                if let AppState::Input = app.state {
                                    if !app.input.is_empty() {
                                        app.history.push(app.input.clone());
                                        app.history_index = None;
                                        app.next();
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                if let AppState::Input = app.state {
                                    app.input.push(c);
                                    app.history_index = None; // Reset history navigation on typing
                                }
                            }
                            KeyCode::Backspace => {
                                if let AppState::Input = app.state {
                                    app.input.pop();
                                    app.history_index = None; // Reset history navigation on typing
                                }
                            }
                            KeyCode::Up => {
                                if let AppState::Input = app.state {
                                    if !app.history.is_empty() {
                                        let idx = app.history_index.unwrap_or(app.history.len());
                                        if idx > 0 {
                                            app.history_index = Some(idx - 1);
                                            app.input = app.history[app.history_index.unwrap()].clone();
                                        }
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if let AppState::Input = app.state {
                                    if let Some(idx) = app.history_index {
                                        if idx + 1 < app.history.len() {
                                            app.history_index = Some(idx + 1);
                                            app.input = app.history[idx + 1].clone();
                                        } else {
                                            app.history_index = None;
                                            app.input.clear();
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for response
        if let Ok(response) = rx.try_recv() {
            app.response = response;
            app.state = AppState::Response;
            app.input.clear();
        }
    }
}

async fn process_query(
    input: String,
    provider: Provider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
    stream: bool,
    config: Config,
    args: Args,
    tx: mpsc::UnboundedSender<String>,
) -> Result<()> {
    let api_key = args.api_key.or(config.api.api_key.clone());
    let timeout = args.timeout.unwrap_or(config.api.timeout);

    let client = ApiClient::new(provider, api_key, timeout)?;

    let messages = vec![Message::text("user".to_string(), input)];

    let request = ChatRequest {
        messages,
        model: Some(model),
        temperature: Some(temperature),
        max_tokens,
        top_p,
        top_k,
        stream: Some(stream),
    };

    let content = if stream {
        let mut full_response = String::new();
        client.send_message_stream(request, |chunk| {
            full_response.push_str(&chunk);
            let _ = tx.send(full_response.clone());
        }).await?
    } else {
        client.send_message(request).await?
    };

    let _ = tx.send(content);
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Main layout: sidebar and main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(size);

    // Sidebar: History
    let history_items: Vec<ListItem> = app
        .history
        .iter()
        .rev()
        .take(10)
        .map(|h| ListItem::new(h.as_str()))
        .collect();
    let history_list = List::new(history_items)
        .block(Block::default().borders(Borders::ALL).title(" History").border_style(Style::default().fg(Color::Cyan)))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));
    f.render_widget(history_list, main_chunks[0]);

    // Right area: settings, response/input, footer
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Settings
            Constraint::Min(1),    // Main area
            Constraint::Length(3), // Input
            Constraint::Length(1), // Footer
        ])
        .split(main_chunks[1]);

    // Settings bar
    let settings = vec![
        Span::styled(format!("Provider: {}", app.provider.name()), Style::default().fg(Color::Green)),
        Span::raw(" | "),
        Span::styled(format!("Model: {}", app.model), Style::default().fg(Color::Blue)),
        Span::raw(" | "),
        Span::styled(format!("Temp: {:.1}", app.temperature), Style::default().fg(Color::Red)),
        Span::raw(" | "),
        Span::styled(format!("Stream: {}", if app.stream { "On" } else { "Off" }), Style::default().fg(Color::Magenta)),
    ];
    let settings_line = Line::from(settings);
    let settings_para = Paragraph::new(settings_line)
        .block(Block::default().borders(Borders::ALL).title(" Settings").border_style(Style::default().fg(Color::White)));
    f.render_widget(settings_para, right_chunks[0]);

    // Main area
    let main_block = Block::default().borders(Borders::ALL).title(" Response").border_style(Style::default().fg(Color::Green));
    f.render_widget(main_block, right_chunks[1]);

    let inner_area = right_chunks[1].inner(Margin { vertical: 1, horizontal: 1 });

    match app.state {
        AppState::Input => {
            let text = "Enter your prompt and press Enter...";
            let para = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray))
                .wrap(Wrap { trim: true });
            f.render_widget(para, inner_area);
        }
        AppState::Processing => {
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" Processing").border_style(Style::default().fg(Color::Yellow)))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(50);
            f.render_widget(gauge, inner_area);
        }
        AppState::Response => {
            let para = Paragraph::new(app.response.as_str())
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            f.render_widget(para, inner_area);
        }
    }

    // Input area
    let input_block = Block::default().borders(Borders::ALL).title(" Input").border_style(Style::default().fg(Color::Blue));
    f.render_widget(input_block, right_chunks[2]);

    let input_area = right_chunks[2].inner(Margin { vertical: 1, horizontal: 1 });
    let input_para = Paragraph::new(app.input.as_str()).style(Style::default().fg(Color::White));
    f.render_widget(input_para, input_area);

    // Footer with shortcuts
    let footer_text = Line::from(vec![
        Span::styled("^T", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Temp  "),
        Span::styled("^S", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Stream  "),
        Span::styled("^H", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Clear Hist  "),
        Span::styled("^R", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Clear Resp  "),
        Span::styled("^Q", Style::default().fg(Color::Black).bg(Color::White)),
        Span::raw(" Quit"),
    ]);
    let footer = Paragraph::new(footer_text)
        .style(Style::default().bg(Color::White).fg(Color::Black));
    f.render_widget(footer, right_chunks[3]);

    // Set cursor
    if let AppState::Input = app.state {
        f.set_cursor_position(Position::new(
            right_chunks[2].x + app.input.len() as u16 + 1,
            right_chunks[2].y + 1,
        ));
    }
}