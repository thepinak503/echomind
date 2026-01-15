use crate::api::{ApiClient, ChatRequest, Message, Provider};
use crate::cli::Args;
use crate::config::Config;
use crate::error::Result;
use crate::platform::system;
use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const MAX_HISTORY: usize = 50;
const MAX_INPUT_LENGTH: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Idle,
    Processing { start_time: Instant },
    Streaming,
    ResponseComplete { duration: Duration },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBubble {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub is_streaming: bool,
}

#[derive(Debug)]
pub struct ChatApp {
    state: AppState,
    messages: VecDeque<MessageBubble>,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    provider: Provider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    stream: bool,
    config: Config,
    args: Args,
}

impl ChatApp {
    pub fn new(config: Config, args: Args) -> Self {
        let provider =
            Provider::from_string(args.provider.as_ref().unwrap_or(&config.api.provider))
                .unwrap_or(Provider::Chat);
        let model = args.model.as_ref().unwrap_or(&config.api.model).clone();
        let temperature = args.temperature.unwrap_or(config.defaults.temperature);
        let max_tokens = args.max_tokens.or(config.defaults.max_tokens);
        let stream = args.stream;
        let messages = load_history().unwrap_or_default();

        Self {
            state: AppState::Idle,
            messages,
            input: String::with_capacity(MAX_INPUT_LENGTH),
            history: Vec::new(),
            history_index: None,
            provider,
            model,
            temperature,
            max_tokens,
            stream,
            config,
            args,
        }
    }

    fn add_message(&mut self, role: String, content: String, streaming: bool) {
        let bubble = MessageBubble {
            role,
            content,
            timestamp: Local::now(),
            is_streaming: streaming,
        };
        self.messages.push_back(bubble);
        if self.messages.len() > MAX_HISTORY * 2 {
            self.messages.pop_front();
        }
        self.save_history();
    }

    fn save_history(&self) {
        if let Ok(json) = serde_json::to_string(&Vec::from(self.messages.clone())) {
            if let Ok(encrypted) = encrypt(json.as_bytes()) {
                if let Some(config_dir) = dirs::config_dir() {
                    let echomind_dir = config_dir.join("echomind");
                    let _ = fs::create_dir_all(&echomind_dir);
                    let path = echomind_dir.join("tui_history.enc");
                    let _ = fs::write(path, encrypted);
                }
            }
        }
    }

    fn handle_command(&mut self) -> bool {
        if !self.input.starts_with('/') {
            return false;
        }

        let parts: Vec<&str> = self.input.split_whitespace().collect();
        let cmd = parts.first().map(|&s| s.to_lowercase()).unwrap_or_default();

        match cmd.as_str() {
            "/help" | "/h" => {
                self.add_message(
                    "Assistant".to_string(),
                    "Commands: /help /clear /model /temp /stream /stats /export /quit".to_string(),
                    false,
                );
            }
            "/clear" | "/c" => {
                self.messages.clear();
                self.input.clear();
                self.state = AppState::Idle;
                self.save_history();
            }
            "/model" if parts.len() > 1 => {
                self.model = parts[1..].join(" ");
                self.add_message(
                    "System".to_string(),
                    format!("Model: {}", self.model),
                    false,
                );
            }
            "/temp" if parts.len() > 1 => {
                if let Ok(t) = parts[1].parse::<f32>() {
                    self.temperature = t.clamp(0.0, 2.0);
                    self.add_message(
                        "System".to_string(),
                        format!("Temp: {:.1}", self.temperature),
                        false,
                    );
                } else {
                    self.add_message("System".to_string(), "Invalid temp".to_string(), false);
                }
            }
            "/stream" => {
                self.stream = !self.stream;
                self.add_message(
                    "System".to_string(),
                    format!("Stream: {}", if self.stream { "ON" } else { "OFF" }),
                    false,
                );
            }
            "/stats" => {
                let msg_count = self.messages.len();
                self.add_message(
                    "System".to_string(),
                    format!("Messages: {}", msg_count),
                    false,
                );
            }
            "/export" => {
                self.add_message("System".to_string(), "Exported".to_string(), false);
            }
            "/quit" | "/q" => {
                self.add_message("System".to_string(), "Goodbye!".to_string(), false);
                return true;
            }
            _ => {
                self.add_message("System".to_string(), "Unknown command".to_string(), false);
            }
        }
        self.input.clear();
        true
    }
}

fn load_history() -> Result<VecDeque<MessageBubble>> {
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("echomind").join("tui_history.enc");
        if path.exists() {
            if let Ok(encrypted) = fs::read(&path) {
                if let Ok(decrypted) = decrypt(&encrypted) {
                    if let Ok(json) = String::from_utf8(decrypted) {
                        if let Ok(msgs) = serde_json::from_str::<Vec<MessageBubble>>(&json) {
                            return Ok(msgs.into());
                        }
                    }
                }
            }
        }
    }
    Ok(VecDeque::new())
}

fn encrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key = UnboundKey::new(&AES_256_GCM, b"echomind_tui_key_32bytes!!")
        .map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".into()))?;
    let key = LessSafeKey::new(key);
    let mut out = data.to_vec();
    let nonce = Nonce::assume_unique_for_key([0u8; 12]);
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut out)
        .map_err(|_| crate::error::EchomindError::ConfigError("Encryption failed".into()))?;
    Ok(out)
}

fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key = UnboundKey::new(&AES_256_GCM, b"echomind_tui_key_32bytes!!")
        .map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".into()))?;
    let key = LessSafeKey::new(key);
    let mut out = data.to_vec();
    let nonce = Nonce::assume_unique_for_key([0u8; 12]);
    key.open_in_place(nonce, Aad::empty(), &mut out)
        .map_err(|_| crate::error::EchomindError::ConfigError("Decryption failed".into()))?;
    Ok(out.to_vec())
}

pub async fn run_tui<B: Backend>(terminal: &mut Terminal<B>, mut app: ChatApp) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let mut response_buffer = String::new();

    loop {
        terminal.draw(|f| draw_ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('c') | KeyCode::Char('q') => break,
                    KeyCode::Char('t') => {
                        app.temperature = match app.temperature {
                            t if t < 0.2 => 0.5,
                            t if t < 0.7 => 1.0,
                            t if t < 1.5 => 1.8,
                            _ => 0.1,
                        };
                    }
                    KeyCode::Char('s') => app.stream = !app.stream,
                    KeyCode::Char('l') => {
                        app.messages.clear();
                        app.state = AppState::Idle;
                        app.save_history();
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            if app.handle_command() {
                                break;
                            }
                            if matches!(
                                app.state,
                                AppState::Idle | AppState::ResponseComplete { .. }
                            ) {
                                let input = app.input.clone();
                                app.add_message("You".to_string(), input.clone(), false);
                                app.history.push(input.clone());
                                app.history_index = None;
                                app.input.clear();
                                app.state = AppState::Processing {
                                    start_time: Instant::now(),
                                };
                                app.add_message(
                                    app.provider.name().to_string(),
                                    String::new(),
                                    true,
                                );

                                let tx_clone = tx.clone();
                                let provider = app.provider.clone();
                                let model = app.model.clone();
                                let temperature = app.temperature;
                                let max_tokens = app.max_tokens;
                                let stream = app.stream;
                                let config = app.config.clone();
                                let args = app.args.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = send_message(
                                        input,
                                        provider,
                                        model,
                                        temperature,
                                        max_tokens,
                                        stream,
                                        config,
                                        args,
                                        tx_clone,
                                    )
                                    .await
                                    {
                                        eprintln!("Error: {}", e);
                                    }
                                });
                            }
                        } else if matches!(app.state, AppState::ResponseComplete { .. }) {
                            app.state = AppState::Idle;
                        }
                    }
                    KeyCode::Char(c)
                        if matches!(
                            app.state,
                            AppState::Idle | AppState::ResponseComplete { .. }
                        ) =>
                    {
                        if app.input.len() < MAX_INPUT_LENGTH {
                            app.input.push(c);
                            app.history_index = None;
                        }
                    }
                    KeyCode::Backspace
                        if matches!(
                            app.state,
                            AppState::Idle | AppState::ResponseComplete { .. }
                        ) =>
                    {
                        app.input.pop();
                        app.history_index = None;
                    }
                    KeyCode::Up
                        if matches!(
                            app.state,
                            AppState::Idle | AppState::ResponseComplete { .. }
                        ) =>
                    {
                        if let Some(idx) = app.history_index {
                            if idx > 0 {
                                app.history_index = Some(idx - 1);
                                app.input = app.history[idx - 1].clone();
                            }
                        } else if !app.history.is_empty() {
                            app.history_index = Some(app.history.len() - 1);
                            app.input = app.history.last().unwrap().clone();
                        }
                    }
                    KeyCode::Down
                        if matches!(
                            app.state,
                            AppState::Idle | AppState::ResponseComplete { .. }
                        ) =>
                    {
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
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        if let Ok(chunk) = rx.try_recv() {
            if chunk.is_empty() {
                if let Some(last) = app.messages.back_mut() {
                    if last.role.as_str() == app.provider.name() {
                        last.is_streaming = false;
                    }
                }
                if let AppState::Processing { start_time } = app.state {
                    app.state = AppState::ResponseComplete {
                        duration: start_time.elapsed(),
                    };
                }
                response_buffer.clear();
            } else {
                response_buffer.push_str(&chunk);
                if let Some(last) = app.messages.back_mut() {
                    if last.role.as_str() == app.provider.name() {
                        last.content = response_buffer.clone();
                    }
                }
            }
        }
    }

    app.save_history();
    Ok(())
}

async fn send_message(
    input: String,
    provider: Provider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    stream: bool,
    config: Config,
    args: Args,
    tx: mpsc::UnboundedSender<String>,
) -> Result<()> {
    let api_key = args.api_key.or(config.api.api_key.clone());
    let timeout = args.timeout.unwrap_or(config.api.timeout);
    let client = ApiClient::new(provider.clone(), api_key, timeout)?;

    let messages = vec![Message::text("user".to_string(), input)];
    let request = ChatRequest {
        messages,
        model: Some(model),
        temperature: Some(temperature),
        max_tokens,
        top_p: None,
        top_k: None,
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

fn draw_ui(f: &mut Frame, app: &mut ChatApp) {
    let size = f.size();
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(4),
            Constraint::Length(3),
        ])
        .split(size);

    draw_header(f, app, main_layout[0]);
    draw_messages(f, app, main_layout[1]);
    draw_input(f, app, main_layout[2]);
    draw_footer(f, app, main_layout[3]);
}

fn draw_header(f: &mut Frame, app: &ChatApp, area: Rect) {
    let platform = system::get_platform();
    let arch = system::get_architecture();

    let header = Block::default()
        .title_alignment(Alignment::Center)
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(17, 17, 17)));

    let content = Line::from(vec![
        Span::styled(" EchoMind ", Style::default().fg(Color::Cyan).bold()),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} ", app.provider.name()),
            Style::default().fg(Color::Green),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} ", app.model),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {:.1} ", app.temperature),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled(
            if app.stream { "ON" } else { "OFF" },
            Style::default().fg(if app.stream { Color::Green } else { Color::Red }),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} {} ", platform, arch),
            Style::default().fg(Color::Blue),
        ),
    ]);

    f.render_widget(
        Paragraph::new(content)
            .block(header)
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_messages(f: &mut Frame, app: &ChatApp, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
        .style(Style::default().bg(Color::Rgb(10, 10, 10)));

    let items: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, msg)| {
            let role_color = match msg.role.as_str() {
                "You" => Color::Cyan,
                "System" => Color::Yellow,
                _ => Color::Green,
            };

            let streaming = if msg.is_streaming { " ..." } else { "" };

            let content = format!(
                "[{}] {}{}\n{}",
                msg.timestamp.format("%H:%M").to_string(),
                msg.role.as_str().bold().fg(role_color),
                streaming,
                msg.content
            );

            ListItem::new(Text::from(content)).style(Style::default().bg(if i % 2 == 0 {
                Color::Rgb(12, 12, 12)
            } else {
                Color::Rgb(15, 15, 15)
            }))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn draw_input(f: &mut Frame, app: &ChatApp, area: Rect) {
    let status_info: (Color, Color, String) = match &app.state {
        AppState::Idle => (Color::Rgb(20, 20, 20), Color::Green, "Ready".to_string()),
        AppState::Processing { .. } => (
            Color::Rgb(30, 25, 0),
            Color::Yellow,
            "Processing...".to_string(),
        ),
        AppState::Streaming => (
            Color::Rgb(0, 25, 30),
            Color::Cyan,
            "Streaming...".to_string(),
        ),
        AppState::ResponseComplete { duration } => (
            Color::Rgb(20, 30, 20),
            Color::Blue,
            format!("Done {:.1}s", duration.as_secs_f32()),
        ),
    };

    let input_text = if app.input.is_empty() {
        "Type message or /help..."
    } else {
        &app.input
    };
    let hint = if app.input.starts_with('/') {
        "/help for commands"
    } else {
        "Enter to send"
    };

    let block = Block::default()
        .title(format!(" Input │ {} ", status_info.2))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(status_info.1))
        .style(Style::default().bg(status_info.0));

    let content = vec![
        Line::from(Span::styled(input_text, Style::default().fg(Color::White))),
        Line::from(Span::styled(
            hint,
            Style::default().fg(Color::DarkGray).italic(),
        )),
    ];

    f.render_widget(Paragraph::new(content).block(block), area);

    if matches!(
        app.state,
        AppState::Idle | AppState::ResponseComplete { .. }
    ) {
        let y = area.y + 1;
        let x = area.x + (app.input.len() as u16).min(area.width - 2) + 1;
        f.set_cursor(x, y);
    }
}

fn draw_footer(f: &mut Frame, app: &ChatApp, area: Rect) {
    let (left, right) = {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        (chunks[0], chunks[1])
    };

    let spinner = match (
        app.state,
        std::time::Instant::now().elapsed().as_millis() / 200 % 4,
    ) {
        (AppState::Processing { .. }, 0) => "◐",
        (AppState::Processing { .. }, 1) => "◓",
        (AppState::Processing { .. }, 2) => "◑",
        (AppState::Processing { .. }, _) => "◒",
        _ => "EchoMind",
    };

    let left_content = Line::from(vec![Span::styled(
        spinner,
        Style::default().fg(Color::Cyan).bold(),
    )]);

    let right_content = Line::from(vec![
        Span::styled("Ctrl+C/Q Exit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+T Temp  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+S Stream  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑↓ History", Style::default().fg(Color::Yellow)),
    ]);

    f.render_widget(
        Paragraph::new(left_content)
            .style(Style::default().bg(Color::Rgb(17, 17, 17)))
            .alignment(Alignment::Left),
        left,
    );
    f.render_widget(
        Paragraph::new(right_content)
            .style(Style::default().bg(Color::Rgb(17, 17, 17)))
            .alignment(Alignment::Right),
        right,
    );
}
