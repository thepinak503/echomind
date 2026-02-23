use crate::api::{ApiClient, ChatRequest, Message, Provider};
use crate::cli::Args;
use crate::config::Config;
use crate::error::Result;
use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame, Terminal,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use textwrap::Options;
use tokio::sync::mpsc;

const MAX_INPUT_LENGTH: usize = 4096;
const SCROLL_LINES: usize = 3;
const PAGE_SCROLL_LINES: usize = 10;

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
    verbose: bool,
    session_log: Vec<String>,
    session_start: DateTime<Local>,
}

impl TuiApp {
    pub fn new(config: Config, args: Args) -> Self {
        let provider =
            Provider::from_string(args.provider.as_ref().unwrap_or(&config.api.provider))
                .unwrap_or(Provider::Chat);
        let model = args.model.as_ref().unwrap_or(&config.api.model).clone();
        let temperature = args.temperature.unwrap_or(config.defaults.temperature);
        let stream = args.stream;
        let verbose = args.verbose;
        let session_start = Local::now();

        let mut messages = VecDeque::new();
        messages.push_back(ChatMessage {
            role: "System".to_string(),
            content: "Welcome to EchoMind TUI! Type /help for commands, or just start chatting."
                .to_string(),
            timestamp: Local::now(),
        });

        Self {
            state: TuiState::Idle,
            messages,
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
            verbose,
            session_log: Vec::new(),
            session_start,
        }
    }

    fn log(&mut self, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("[{}] {}", timestamp, message);
        self.session_log.push(log_entry);
    }

    fn log_error(&mut self, error: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("[{}] ERROR: {}", timestamp, error);
        self.session_log.push(log_entry);
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
                let help_text = r#"Available Commands:
/help, /h     - Show this help message
/clear, /c    - Clear chat history
/model <name> - Change AI model
/temp <value> - Set temperature (0.0-2.0)
/stream       - Toggle streaming mode
/verbose      - Toggle verbose mode
/logs         - Show session logs
/quit, /q     - Exit TUI

Keyboard Shortcuts:
Ctrl+C, Ctrl+Q - Exit
Ctrl+T         - Cycle temperature
Ctrl+S         - Toggle streaming
Ctrl+L         - Clear screen
Enter          - Send message
Up/Down        - Navigate history
Page Up/Down   - Scroll messages
Home/End       - Jump to top/bottom
Mouse Wheel    - Scroll messages
Esc            - Exit

Header Info:
  TEMP:X.X   - Current temperature
  STREAM:ON/OFF - Streaming status"#;
                self.add_message("System".to_string(), help_text.to_string());
            }
            "/clear" | "/c" => {
                self.messages.clear();
                self.input.clear();
                self.state = TuiState::Idle;
            }
            "/model" if parts.len() > 1 => {
                self.model = parts[1..].join(" ");
                self.add_message("System".to_string(), format!("Model: {}", self.model));
                self.log(&format!("Model changed to: {}", self.model));
            }
            "/temp" if parts.len() > 1 => {
                if let Ok(t) = parts[1].parse::<f32>() {
                    self.temperature = t.clamp(0.0, 2.0);
                    self.add_message(
                        "System".to_string(),
                        format!("Temp: {:.1}", self.temperature),
                    );
                    self.log(&format!("Temperature set to: {:.1}", self.temperature));
                }
            }
            "/stream" => {
                self.stream = !self.stream;
                self.add_message(
                    "System".to_string(),
                    format!("Stream: {}", if self.stream { "ON" } else { "OFF" }),
                );
                self.log(&format!("Stream mode: {}", if self.stream { "ON" } else { "OFF" }));
            }
            "/logs" => {
                if self.session_log.is_empty() {
                    self.add_message("System".to_string(), "No session logs available.".to_string());
                } else {
                    let logs = self.session_log.join("\n");
                    self.add_message("System".to_string(), format!("Session Logs:\n{}", logs));
                }
            }
            "/verbose" => {
                self.verbose = !self.verbose;
                self.add_message(
                    "System".to_string(),
                    format!("Verbose mode: {}", if self.verbose { "ON" } else { "OFF" }),
                );
                self.log(&format!("Verbose mode: {}", if self.verbose { "ON" } else { "OFF" }));
            }
            "/quit" | "/q" => {
                self.add_message("System".to_string(), "Goodbye!".to_string());
                self.input.clear();
                return true;
            }
            _ => {
                self.add_message("System".to_string(), format!("Unknown command: {}. Type /help for commands.", cmd));
            }
        }
        self.input.clear();
        false
    }
}

fn _encrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key = UnboundKey::new(&AES_256_GCM, b"echomind_tui_key_32bytes!!")
        .map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".into()))?;
    let key = LessSafeKey::new(key);
    let mut out = data.to_vec();
    let nonce = Nonce::assume_unique_for_key([0u8; 12]);
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut out)
        .map_err(|_| crate::error::EchomindError::ConfigError("Encryption failed".into()))?;
    Ok(out)
}

fn _decrypt(data: &[u8]) -> Result<Vec<u8>> {
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
    let mut total_lines = 0;
    let mut vertical_scroll = 0usize;

    loop {
        terminal.draw(|f| {
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

            draw_header(f, &app, chunks[0]);
            let (lines, scroll) = draw_messages(f, &app, chunks[1], vertical_scroll);
            total_lines = lines;
            draw_input(f, &app, chunks[2]);
            draw_footer(f, &app, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
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
                                vertical_scroll = 0;
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
                                    app.log(&format!("User message: {}", if input.len() > 50 { &input[..50] } else { &input }));
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

                                    tokio::spawn(send_message(
                                        input,
                                        provider,
                                        model,
                                        temperature,
                                        stream,
                                        config,
                                        args,
                                        tx_clone,
                                    ));
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
                            KeyCode::PageUp => {
                                let visible = 20usize;
                                let max_scroll = total_lines.saturating_sub(visible);
                                vertical_scroll = vertical_scroll.saturating_add(PAGE_SCROLL_LINES).min(max_scroll);
                            }
                            KeyCode::PageDown => {
                                vertical_scroll = vertical_scroll.saturating_sub(PAGE_SCROLL_LINES);
                            }
                            KeyCode::Home => {
                                vertical_scroll = total_lines;
                            }
                            KeyCode::End => {
                                vertical_scroll = 0;
                            }
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                }
                Event::Mouse(MouseEvent { kind, .. }) => {
                    match kind {
                        MouseEventKind::ScrollUp => {
                            let visible = 20usize;
                            let max_scroll = total_lines.saturating_sub(visible);
                            vertical_scroll = vertical_scroll.saturating_add(SCROLL_LINES).min(max_scroll);
                        }
                        MouseEventKind::ScrollDown => {
                            vertical_scroll = vertical_scroll.saturating_sub(SCROLL_LINES);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if let Ok(chunk) = rx.try_recv() {
            if chunk.is_empty() {
                if app.state == TuiState::Processing {
                    app.state = TuiState::Complete;
                    if let Some(start) = processing_start {
                        let duration = start.elapsed();
                        app.last_duration = Some(duration);
                        if app.verbose {
                            app.log(&format!("Response completed in {:.2}s", duration.as_secs_f32()));
                        }
                    }
                    processing_start = None;
                }
            } else if chunk.starts_with("Error:") {
                let error_msg = chunk.strip_prefix("Error:").unwrap_or(&chunk);
                app.log_error(error_msg);
                response_content.push_str(&chunk);
                if let Some(last) = app.messages.back_mut() {
                    if last.role == app.provider.name() {
                        last.content = response_content.clone();
                    }
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

    save_and_print_session_log(&app);

    Ok(())
}

fn save_and_print_session_log(app: &TuiApp) {
    if app.session_log.is_empty() && !app.verbose {
        return;
    }

    let log_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("echomind")
        .join("logs");

    if let Err(e) = fs::create_dir_all(&log_dir) {
        eprintln!("\nFailed to create log directory: {}", e);
        return;
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let log_file = log_dir.join(format!("echomind_session_{}.log", timestamp));

    let mut file_content = String::new();
    file_content.push_str(&format!("EchoMind TUI Session Log\n"));
    file_content.push_str(&format!("Started: {}\n", app.session_start.format("%Y-%m-%d %H:%M:%S")));
    file_content.push_str(&format!("Ended: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    file_content.push_str(&format!("Provider: {}\n", app.provider.name()));
    file_content.push_str(&format!("Model: {}\n", app.model));
    file_content.push_str(&format!("Temperature: {:.1}\n", app.temperature));
    file_content.push_str(&format!("Stream: {}\n", if app.stream { "ON" } else { "OFF" }));
    file_content.push_str(&format!("Verbose: {}\n", if app.verbose { "ON" } else { "OFF" }));
    file_content.push_str("\n--- Session Log ---\n\n");

    for entry in &app.session_log {
        file_content.push_str(entry);
        file_content.push('\n');
    }

    file_content.push_str("\n--- Chat History ---\n\n");
    for msg in &app.messages {
        let timestamp = msg.timestamp.format("%H:%M:%S");
        file_content.push_str(&format!("[{}] {}: {}\n\n", timestamp, msg.role, msg.content));
    }

    match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&log_file)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(file_content.as_bytes()) {
                eprintln!("\nFailed to write log file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("\nFailed to create log file: {}", e);
            return;
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("EchoMind TUI Session Ended");
    println!("{}", "=".repeat(60));
    println!("Session Duration: {}", {
        let duration = (Local::now() - app.session_start).num_seconds();
        let mins = duration / 60;
        let secs = duration % 60;
        format!("{}m {}s", mins, secs)
    });
    println!("Messages: {}", app.messages.len());
    println!("Log File: {}", log_file.display());
    println!("{}", "=".repeat(60));

    if !app.session_log.is_empty() {
        println!("\nSession Log:");
        println!("{}", "-".repeat(40));
        for entry in &app.session_log {
            println!("{}", entry);
        }
        println!("{}", "-".repeat(40));
    }
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
) {
    let api_key = args.api_key.or(config.api.api_key.clone());
    let timeout = args.timeout.unwrap_or(config.api.timeout);

    let client = match ApiClient::new(provider.clone(), api_key, timeout) {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(format!("Error: {}", e));
            let _ = tx.send(String::new());
            return;
        }
    };

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

    let result = if stream {
        client
            .send_message_stream(request, |chunk| {
                let _ = tx.send(chunk.to_string());
            })
            .await
    } else {
        match client.send_message(request).await {
            Ok(content) => {
                let _ = tx.send(content);
                Ok(String::new())
            }
            Err(e) => Err(e),
        }
    };

    if let Err(e) = result {
        let _ = tx.send(format!("Error: {}", e));
    }

    let _ = tx.send(String::new());
}

fn draw_header(f: &mut Frame, app: &TuiApp, area: Rect) {
    let header = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Rgb(17, 17, 17)));

    let stream_indicator = if app.stream { "STREAM:ON" } else { "STREAM:OFF" };
    let stream_color = if app.stream { Color::Green } else { Color::Red };

    let content = Line::from(vec![
        Span::styled(" EchoMind ", Style::default().fg(Color::Cyan).bold()),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.provider.name(),
            Style::default().fg(Color::Green),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &app.model,
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "TEMP:",
            Style::default().fg(Color::Magenta),
        ),
        Span::styled(
            format!("{:.1}", app.temperature),
            Style::default().fg(Color::White),
        ),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            stream_indicator,
            Style::default().fg(stream_color),
        ),
        Span::styled(
            format!(" | v{}", env!("CARGO_PKG_VERSION")),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    f.render_widget(
        Paragraph::new(content)
            .block(header)
            .alignment(Alignment::Center),
        area,
    );
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return text.lines().map(String::from).collect();
    }

    let options = Options::new(width);
    
    let mut wrapped_lines = Vec::new();
    
    for line in text.lines() {
        let wrapped = textwrap::wrap(line, &options);
        if wrapped.is_empty() {
            wrapped_lines.push(String::new());
        } else {
            for wrapped_line in wrapped {
                wrapped_lines.push(wrapped_line.into_owned());
            }
        }
    }
    
    wrapped_lines
}

fn draw_messages(f: &mut Frame, app: &TuiApp, area: Rect, vertical_scroll: usize) -> (usize, usize) {
    let content_width = area.width.saturating_sub(6) as usize;
    let visible_height = area.height.saturating_sub(2) as usize;
    
    let mut all_lines: Vec<(String, Color, Color)> = Vec::new();
    
    for (i, msg) in app.messages.iter().enumerate() {
        let role_color = match msg.role.as_str() {
            "You" => Color::Cyan,
            "System" => Color::Yellow,
            _ => Color::Green,
        };
        
        let bg_color = if i % 2 == 0 {
            Color::Rgb(12, 12, 12)
        } else {
            Color::Rgb(15, 15, 15)
        };

        let header = format!("[{}] {}", msg.timestamp.format("%H:%M"), msg.role);
        all_lines.push((header, role_color, bg_color));

        let wrapped_content = wrap_text(&msg.content, content_width);
        for line in wrapped_content {
            all_lines.push((line, Color::White, bg_color));
        }
        
        all_lines.push((String::new(), Color::White, bg_color));
    }

    let total_lines = all_lines.len();
    
    let max_scroll = total_lines.saturating_sub(visible_height);
    let vertical_scroll = vertical_scroll.min(max_scroll);
    
    let scroll_from_bottom = max_scroll.saturating_sub(vertical_scroll);
    
    let visible_lines: Vec<_> = all_lines
        .iter()
        .skip(scroll_from_bottom)
        .take(visible_height)
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 60)))
        .style(Style::default().bg(Color::Rgb(10, 10, 10)));

    let title = if total_lines > visible_height {
        let current_pos = visible_height + scroll_from_bottom;
        format!(" Messages [{}/{}] ", current_pos.min(total_lines), total_lines)
    } else {
        " Messages ".to_string()
    };

    let items: Vec<ListItem> = visible_lines
        .iter()
        .map(|(text, fg_color, bg_color)| {
            ListItem::new(Text::from(text.clone()))
                .style(Style::default().fg(*fg_color).bg(*bg_color))
        })
        .collect();

    let list = List::new(items)
        .block(block.title(title))
        .style(Style::default().fg(Color::White));
    f.render_widget(list, area);

    if total_lines > visible_height && visible_height > 0 {
        let scrollbar_area = Rect::new(
            area.x + area.width - 1,
            area.y + 1,
            1,
            area.height.saturating_sub(2),
        );
        
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(scroll_from_bottom)
            .viewport_content_length(visible_height);
        
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    (total_lines, vertical_scroll)
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
        .title(format!(" Input | {} ", status))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));

    let content = vec![
        Line::from(Span::styled(
            display_text,
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Enter: send | Up/Down: history | PgUp/PgDn | Mouse: scroll | /help",
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

fn draw_footer(f: &mut Frame, _app: &TuiApp, area: Rect) {
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled("Ctrl+C/Q Exit  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+T Temp  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Ctrl+S Stream  ", Style::default().fg(Color::DarkGray)),
        Span::styled("Mouse Wheel: Scroll", Style::default().fg(Color::Yellow)),
    ])])
    .style(Style::default().bg(Color::Rgb(17, 17, 17)))
    .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
