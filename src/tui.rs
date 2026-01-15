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
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const MAX_INPUT_LENGTH: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TuiState {
    Idle,
    Processing,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: DateTime<Local>,
}

pub struct TuiApp {
    state: TuiState,
    messages: VecDeque<ChatMessage>,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    provider: Provider,
    model: String,
    temperature: f32,
    stream: bool,
    config: Config,
    args: Args,
    last_duration: Option<Duration>,
}

impl TuiApp {
    pub fn new(config: Config, args: Args) -> Self {
        let provider =
            Provider::from_string(args.provider.as_ref().unwrap_or(&config.api.provider))
                .unwrap_or(Provider::Chat);
        let model = args.model.as_ref().unwrap_or(&config.api.model).clone();
        let temperature = args.temperature.unwrap_or(config.defaults.temperature);
        let stream = args.stream;

        Self {
            state: TuiState::Idle,
            messages: VecDeque::new(),
            input: String::with_capacity(MAX_INPUT_LENGTH),
            history: Vec::new(),
            history_index: None,
            provider,
            model,
            temperature,
            stream,
            config,
            args,
            last_duration: None,
        }
    }

    fn add_message(&mut self, role: String, content: String) {
        self.messages.push_back(ChatMessage {
            role,
            content,
            timestamp: Local::now(),
        });
        if self.messages.len() > 100 {
            self.messages.pop_front();
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
                    "Commands: /help /clear /model /temp /stream /quit".to_string(),
                );
            }
            "/clear" | "/c" => {
                self.messages.clear();
                self.input.clear();
                self.state = TuiState::Idle;
            }
            "/model" if parts.len() > 1 => {
                self.model = parts[1..].join(" ");
                self.add_message("System".to_string(), format!("Model: {}", self.model));
            }
            "/temp" if parts.len() > 1 => {
                if let Ok(t) = parts[1].parse::<f32>() {
                    self.temperature = t.clamp(0.0, 2.0);
                    self.add_message(
                        "System".to_string(),
                        format!("Temp: {:.1}", self.temperature),
                    );
                }
            }
            "/stream" => {
                self.stream = !self.stream;
                self.add_message(
                    "System".to_string(),
                    format!("Stream: {}", if self.stream { "ON" } else { "OFF" }),
                );
            }
            "/quit" | "/q" => {
                self.add_message("System".to_string(), "Goodbye!".to_string());
                return true;
            }
            _ => {
                self.add_message("System".to_string(), "Unknown command".to_string());
            }
        }
        self.input.clear();
        true
    }
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

pub async fn run_tui<B: Backend>(terminal: &mut Terminal<B>, mut app: TuiApp) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let mut response_content = String::new();
    let mut processing_start: Option<Instant> = None;

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(50))? {
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
                            app.state = TuiState::Idle;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Enter => {
                            if app.state == TuiState::Processing {
                                continue;
                            }
                            if !app.input.is_empty() {
                                if app.handle_command() {
                                    break;
                                }
                                let input = app.input.clone();
                                app.add_message("You".to_string(), input.clone());
                                app.history.push(input.clone());
                                app.history_index = None;
                                app.input.clear();
                                app.state = TuiState::Processing;
                                processing_start = Some(Instant::now());
                                app.add_message(app.provider.name().to_string(), String::new());
                                response_content.clear();

                                let tx_clone = tx.clone();
                                let provider = app.provider.clone();
                                let model = app.model.clone();
                                let temperature = app.temperature;
                                let stream = app.stream;
                                let config = app.config.clone();
                                let args = app.args.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = send_message(
                                        input,
                                        provider,
                                        model,
                                        temperature,
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
                        }
                        KeyCode::Char(c) => {
                            if app.state != TuiState::Processing
                                && app.input.len() < MAX_INPUT_LENGTH
                            {
                                app.input.push(c);
                                app.history_index = None;
                            }
                        }
                        KeyCode::Backspace => {
                            if app.state != TuiState::Processing {
                                app.input.pop();
                                app.history_index = None;
                            }
                        }
                        KeyCode::Up => {
                            if app.state != TuiState::Processing {
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
                        }
                        KeyCode::Down => {
                            if app.state != TuiState::Processing {
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
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        if let Ok(chunk) = rx.try_recv() {
            if chunk.is_empty() {
                if app.state == TuiState::Processing {
                    app.state = TuiState::Complete;
                    if let Some(start) = processing_start {
                        app.last_duration = Some(start.elapsed());
                    }
                    processing_start = None;
                }
            } else {
                response_content.push_str(&chunk);
                if let Some(last) = app.messages.back_mut() {
                    if last.role == app.provider.name() {
                        last.content = response_content.clone();
                    }
                }
            }
        }
    }

    Ok(())
}

async fn send_message(
    input: String,
    provider: Provider,
    model: String,
    temperature: f32,
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
        max_tokens: None,
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

fn draw(f: &mut Frame, app: &TuiApp) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    draw_messages(f, app, chunks[1]);
    draw_input(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);
}

fn draw_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let header = Block::default()
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
    ]);

    f.render_widget(
        Paragraph::new(content)
            .block(header)
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_messages(f: &mut Frame, app: &TuiApp, area: Rect) {
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

            let content = format!(
                "[{}] {}\n{}",
                msg.timestamp.format("%H:%M").to_string(),
                msg.role.as_str().bold().fg(role_color),
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

fn draw_input(f: &mut Frame, app: &TuiApp, area: Rect) {
    let border_color = match app.state {
        TuiState::Idle => Color::Green,
        TuiState::Processing => Color::Yellow,
        TuiState::Complete => Color::Blue,
    };

    let status = match app.state {
        TuiState::Idle => "Ready".to_string(),
        TuiState::Processing => "Processing...".to_string(),
        TuiState::Complete => {
            if let Some(d) = app.last_duration {
                format!("Done {:.1}s", d.as_secs_f32())
            } else {
                "Done".to_string()
            }
        }
    };

    let display_text = if app.input.is_empty() {
        "Type message..."
    } else {
        &app.input
    };

    let block = Block::default()
        .title(format!(" Input │ {} ", status))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));

    let content = vec![
        Line::from(Span::styled(
            display_text,
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Enter to send | ↑↓ history | /help for commands",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(content).block(block), area);

    if app.state != TuiState::Processing {
        let y = area.y + 1;
        let x = area.x + (app.input.len() as u16).min(area.width - 2) + 1;
        f.set_cursor(x, y);
    }
}

fn draw_footer(f: &mut Frame, app: &TuiApp, area: Rect) {
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("Ctrl+C/Q Exit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+T Temp  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+S Stream  ", Style::default().fg(Color::DarkGray)),
        Span::styled("↑↓ History", Style::default().fg(Color::Yellow)),
    ])])
    .style(Style::default().bg(Color::Rgb(17, 17, 17)))
    .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
