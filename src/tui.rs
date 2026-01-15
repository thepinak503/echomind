use crate::api::{ApiClient, ChatRequest, Message, Provider};
use crate::cli::Args;
use crate::config::Config;
use crate::error::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
enum AppState {
    Input,
    Processing,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMessage {
    role: String,
    content: String,
    timestamp: String,
}

#[derive(Debug)]
pub struct App {
    state: AppState,
    input: String,
    messages: Vec<AppMessage>,
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
    scroll_offset: u16,
    processing_spinner: usize,
    last_response_time: Option<Instant>,
}

impl App {
    pub fn new(config: Config, args: Args) -> Self {
        let provider =
            Provider::from_string(args.provider.as_ref().unwrap_or(&config.api.provider))
                .unwrap_or(Provider::Chat);
        let model = args.model.as_ref().unwrap_or(&config.api.model).clone();
        let temperature = args.temperature.unwrap_or(config.defaults.temperature);
        let max_tokens = args.max_tokens.or(config.defaults.max_tokens);
        let top_p = args.top_p.or(config.defaults.top_p);
        let top_k = args.top_k.or(config.defaults.top_k);
        let stream = args.stream;
        let messages = load_chat_history(&config).unwrap_or_default();

        Self {
            state: AppState::Input,
            input: String::new(),
            messages,
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
            scroll_offset: 0,
            processing_spinner: 0,
            last_response_time: None,
        }
    }

    fn add_message(&mut self, role: String, content: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.messages.push(AppMessage {
            role,
            content,
            timestamp,
        });
        let _ = self.save_messages();
    }

    fn save_messages(&self) -> Result<()> {
        let json = serde_json::to_string(&self.messages)?;
        let encrypted = encrypt(json.as_bytes())?;
        let config_dir = dirs::config_dir()
            .ok_or(crate::error::EchomindError::ConfigError(
                "No config dir".to_string(),
            ))?
            .join("echomind");
        fs::create_dir_all(&config_dir)?;
        let path = config_dir.join("chat_history.enc");
        fs::write(path, encrypted)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn get_chat_text(&self) -> String {
        self.messages
            .iter()
            .map(|m| format!("[{}] {}: {}", m.timestamp, m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn process_command(&mut self) -> bool {
        if self.input.starts_with('/') {
            let parts: Vec<&str> = self.input.split_whitespace().collect();
            match parts.get(0).map(|&s| s) {
                Some("/help") | Some("/h") => {
                    self.add_message(
                        "System".to_string(),
                        "Commands: /help, /clear, /export, /settings, /model <name>, /temp <value>"
                            .to_string(),
                    );
                    self.input.clear();
                    true
                }
                Some("/clear") | Some("/c") => {
                    self.messages.clear();
                    self.input.clear();
                    let _ = self.save_messages();
                    true
                }
                Some("/settings") => {
                    let settings = format!(
                        "Provider: {} | Model: {} | Temp: {:.2} | Max Tokens: {} | Stream: {}",
                        self.provider.name(),
                        self.model,
                        self.temperature,
                        self.max_tokens.unwrap_or(0),
                        if self.stream { "On" } else { "Off" }
                    );
                    self.add_message("System".to_string(), settings);
                    self.input.clear();
                    true
                }
                Some("/model") if parts.len() > 1 => {
                    self.model = parts[1..].join(" ");
                    self.add_message(
                        "System".to_string(),
                        format!("Model changed to: {}", self.model),
                    );
                    self.input.clear();
                    true
                }
                Some("/temp") if parts.len() > 1 => {
                    if let Ok(temp) = parts[1].parse::<f32>() {
                        self.temperature = temp.clamp(0.0, 2.0);
                        self.add_message(
                            "System".to_string(),
                            format!("Temperature set to: {:.2}", self.temperature),
                        );
                    } else {
                        self.add_message(
                            "System".to_string(),
                            "Invalid temperature value".to_string(),
                        );
                    }
                    self.input.clear();
                    true
                }
                Some("/export") => {
                    if let Ok(()) = self.save_messages() {
                        self.add_message("System".to_string(), "Chat history saved!".to_string());
                    }
                    self.input.clear();
                    true
                }
                Some("/count") => {
                    let count = self.messages.len();
                    self.add_message("System".to_string(), format!("Total messages: {}", count));
                    self.input.clear();
                    true
                }
                _ => {
                    self.add_message(
                        "System".to_string(),
                        "Unknown command. Type /help for available commands.".to_string(),
                    );
                    self.input.clear();
                    true
                }
            }
        } else {
            false
        }
    }
}

#[allow(dead_code)]
fn save_chat_history(app: &App) -> Result<()> {
    app.save_messages()
}

fn load_chat_history(_config: &Config) -> Result<Vec<AppMessage>> {
    let config_dir = dirs::config_dir()
        .ok_or(crate::error::EchomindError::ConfigError(
            "No config dir".to_string(),
        ))?
        .join("echomind");
    let path = config_dir.join("chat_history.enc");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let encrypted = fs::read(path)?;
    let decrypted = decrypt(&encrypted)?;
    let json = String::from_utf8(decrypted)?;
    let messages: Vec<AppMessage> = serde_json::from_str(&json)?;
    Ok(messages)
}

fn encrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = b"01234567890123456789012345678901"; // 32 bytes for AES-256
    let key = UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".to_string()))?;
    let key = LessSafeKey::new(key);
    let mut in_out = data.to_vec();
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| crate::error::EchomindError::ConfigError("Encryption failed".to_string()))?;
    Ok(in_out)
}

fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = b"01234567890123456789012345678901";
    let key = UnboundKey::new(&AES_256_GCM, key_bytes)
        .map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".to_string()))?;
    let key = LessSafeKey::new(key);
    let mut in_out = data.to_vec();
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let decrypted = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| crate::error::EchomindError::ConfigError("Decryption failed".to_string()))?;
    Ok(decrypted.to_vec())
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        // Update spinner animation
        if matches!(app.state, AppState::Processing) {
            app.processing_spinner = (app.processing_spinner + 1) % 4;
        }

        // Check for new responses
        if let Ok(response) = rx.try_recv() {
            if response.is_empty() {
                // Signal completion
                if !app.messages.is_empty() {
                    if let Some(last) = app.messages.last_mut() {
                        if last.role == app.provider.name() {
                            app.input.clear();
                            app.state = AppState::Response;
                            app.last_response_time = Some(Instant::now());
                        }
                    }
                }
            } else {
                // Append to response (streaming)
                if let Some(last) = app.messages.last_mut() {
                    if last.role == app.provider.name() {
                        last.content.push_str(&response);
                    }
                }
            }
        }

        // Handle events with timeout
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') | KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('t') => {
                            // Cycle temperature
                            app.temperature = match app.temperature {
                                temp if temp < 0.2 => 0.5,
                                temp if temp < 0.7 => 1.0,
                                temp if temp < 1.5 => 1.8,
                                _ => 0.1,
                            };
                        }
                        KeyCode::Char('s') => {
                            // Toggle stream
                            app.stream = !app.stream;
                        }
                        KeyCode::Char('l') => {
                            // Clear all messages
                            app.messages.clear();
                            app.input.clear();
                            app.state = AppState::Input;
                            let _ = app.save_messages();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Enter => {
                            if let AppState::Input = app.state {
                                if !app.input.is_empty() {
                                    // Check if it's a command
                                    if app.process_command() {
                                        // Command was processed
                                    } else {
                                        let input = app.input.clone();
                                        app.add_message("You".to_string(), input.clone());
                                        app.history.push(input.clone());
                                        app.history_index = None;
                                        app.input.clear(); // Clear input immediately
                                        app.state = AppState::Processing;

                                        // Add placeholder for response
                                        app.add_message(
                                            app.provider.name().to_string(),
                                            String::new(),
                                        );

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

                                        tokio::task::spawn(async move {
                                            if let Err(e) = process_query(
                                                input,
                                                provider,
                                                model,
                                                temperature,
                                                max_tokens,
                                                top_p,
                                                top_k,
                                                stream,
                                                config,
                                                args,
                                                tx_process.clone(),
                                            )
                                            .await
                                            {
                                                eprintln!("Error: {:?}", e);
                                            }
                                            let _ = tx_process.send(String::new());
                                        });
                                    }
                                }
                            } else if let AppState::Response = app.state {
                                app.state = AppState::Input;
                                app.input.clear(); // Clear input when transitioning back
                            }
                        }
                        KeyCode::Char(c) => {
                            if let AppState::Input = app.state {
                                app.input.push(c);
                                app.history_index = None;
                            }
                        }
                        KeyCode::Backspace => {
                            if let AppState::Input = app.state {
                                app.input.pop();
                                app.history_index = None;
                            }
                        }
                        KeyCode::Up => {
                            if let AppState::Input = app.state {
                                if !app.history.is_empty() {
                                    let idx = app.history_index.unwrap_or(app.history.len());
                                    if idx > 0 {
                                        app.history_index = Some(idx - 1);
                                        app.input = app.history[idx - 1].clone();
                                    }
                                }
                            } else {
                                app.scroll_offset = app.scroll_offset.saturating_add(1);
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
                            } else {
                                app.scroll_offset = app.scroll_offset.saturating_sub(1);
                            }
                        }
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
            }
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

    if stream {
        client
            .send_message_stream(request, |chunk| {
                let _ = tx.send(chunk.to_string());
            })
            .await?;
    } else {
        let content = client.send_message(request).await?;
        let _ = tx.send(content);
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(4),    // Chat area
            Constraint::Length(4), // Input area
            Constraint::Length(2), // Footer
        ])
        .split(size);

    // Header with status
    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let header_content = vec![
        Line::from(vec![
            Span::styled(
                "  Echomind TUI",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("Provider: {}", app.provider.name()),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("Model: {}", app.model),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("Temperature: {:.2}", app.temperature),
                Style::default().fg(Color::Magenta),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("Stream: {}", if app.stream { "On " } else { "Off" }),
                Style::default().fg(if app.stream { Color::Green } else { Color::Red }),
            ),
            Span::raw("  |  "),
            Span::styled("Messages: ", Style::default().fg(Color::White)),
            Span::raw(format!("{}", app.messages.len())),
        ]),
    ];

    let header = Paragraph::new(header_content).block(header_block);
    f.render_widget(header, chunks[0]);

    // Chat area with better formatting
    let chat_block = Block::default()
        .borders(Borders::ALL)
        .title(" Chat History ")
        .border_style(Style::default().fg(Color::Cyan))
        .title_alignment(ratatui::layout::Alignment::Left);

    let chat_lines: Vec<Line> = app
        .messages
        .iter()
        .map(|m| {
            let role_style = match m.role.as_str() {
                "You" => Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
                "System" => Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
                _ => Style::default().fg(Color::Green),
            };

            let prefix = format!("[{}] {}: ", m.timestamp, m.role);
            let full_text = format!("{}{}", prefix, m.content);
            Line::from(vec![Span::styled(full_text, role_style)])
        })
        .collect();

    let chat_para = Paragraph::new(chat_lines)
        .block(chat_block)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset, 0));
    f.render_widget(chat_para, chunks[1]);

    // Input area
    let input_style = match app.state {
        AppState::Input => Style::default().fg(Color::Green),
        AppState::Processing => Style::default().fg(Color::Yellow),
        AppState::Response => Style::default().fg(Color::Gray),
    };
    let input_title = match app.state {
        AppState::Input => " Input (Ready) ",
        AppState::Processing => " Processing... ",
        AppState::Response => " Response Complete ",
    };

    let hint_text = if app.input.starts_with('/') {
        "Command mode (type /help for available commands)"
    } else {
        "Regular message mode (start with / for commands)"
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(input_title)
        .border_style(input_style);

    let input_content = vec![
        Line::from(Span::raw(&app.input)),
        Line::from(Span::styled(
            hint_text,
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .fg(Color::Gray),
        )),
    ];

    let input_para = Paragraph::new(input_content)
        .block(input_block)
        .style(Style::default().fg(Color::White));
    f.render_widget(input_para, chunks[2]);

    if let AppState::Input | AppState::Processing = app.state {
        f.set_cursor(chunks[2].x + (app.input.len() as u16) + 1, chunks[2].y + 1);
    }

    // Footer with help
    let spinner = match app.processing_spinner {
        0 => "◐",
        1 => "◓",
        2 => "◑",
        _ => "◒",
    };

    let footer_lines = if matches!(app.state, AppState::Processing) {
        vec![Line::from(format!("{} Processing... Please wait", spinner))]
    } else if let Some(time) = app.last_response_time {
        let elapsed = time.elapsed().as_secs();
        vec![
            Line::from(format!("Response completed in {}s", elapsed)),
            Line::from("^C/^Q: Quit  |  ^T: Temp  |  ^S: Stream  |  ^L: Clear  |  ↑↓: History"),
        ]
    } else {
        vec![
            Line::from("Ready for input"),
            Line::from("Enter message or /help for commands  |  ^C/^Q: Quit  |  ↑↓: History  |  Tab: Focus"),
        ]
    };

    let footer = Paragraph::new(footer_lines).style(
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    );
    f.render_widget(footer, chunks[3]);
}
