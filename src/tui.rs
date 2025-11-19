use crate::api::{Provider};
use crate::cli::Args;
use crate::config::Config;
use crate::error::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    // execute,
    // terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend},
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use serde_json;
use std::fs;
use std::io;
use tokio::sync::mpsc;
// use tokio::task;

#[derive(Debug, Clone)]
enum AppState {
    Input,
    Processing,
    Response,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Message {
    sender: String,
    content: String,
    timestamp: u64,
}

#[derive(Debug)]
pub struct App {
    state: AppState,
    input: String,
    messages: Vec<Message>,
    provider: Provider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    stream: bool,
    history: Vec<String>,
    history_index: Option<usize>,
    config: Config,
    args: Args,
}

impl App {
    pub fn new(config: Config, args: Args) -> Self {
        let provider = Provider::from_string(args.provider.as_ref().unwrap_or(&config.api.provider)).unwrap_or(Provider::Chat);
        let model = args.model.as_ref().unwrap_or(&config.api.model).clone();
        let temperature = args.temperature.unwrap_or(config.defaults.temperature);
        let max_tokens = args.max_tokens.or(config.defaults.max_tokens);
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
            stream,
            history: Vec::new(),
            history_index: None,
            config,
            args,
        }

    }

}

fn save_chat_history(app: &App) -> Result<()> {
    let json = serde_json::to_string(&app.messages)?;
    let encrypted = encrypt(json.as_bytes())?;
    let config_dir = dirs::config_dir().ok_or(crate::error::EchomindError::ConfigError("No config dir".to_string()))?.join("echomind");
    fs::create_dir_all(&config_dir)?;
    let path = config_dir.join("chat_history.enc");
    fs::write(path, encrypted)?;
    Ok(())
}

fn load_chat_history(_config: &Config) -> Result<Vec<Message>> {
    let config_dir = dirs::config_dir().ok_or(crate::error::EchomindError::ConfigError("No config dir".to_string()))?.join("echomind");
    let path = config_dir.join("chat_history.enc");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let encrypted = fs::read(path)?;
    let decrypted = decrypt(&encrypted)?;
    let json = String::from_utf8(decrypted)?;
    let messages: Vec<Message> = serde_json::from_str(&json)?;
    Ok(messages)
}

fn encrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = b"01234567890123456789012345678901"; // 32 bytes for AES-256
    let key = UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".to_string()))?;
    let key = LessSafeKey::new(key);
    let mut in_out = data.to_vec();
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out).map_err(|_| crate::error::EchomindError::ConfigError("Encryption failed".to_string()))?;
    Ok(in_out)
}

fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
    let key_bytes = b"01234567890123456789012345678901";
    let key = UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|_| crate::error::EchomindError::ConfigError("Invalid key".to_string()))?;
    let key = LessSafeKey::new(key);
    let mut in_out = data.to_vec();
    let nonce_bytes = [0u8; 12];
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let decrypted = key.open_in_place(nonce, Aad::empty(), &mut in_out).map_err(|_| crate::error::EchomindError::ConfigError("Decryption failed".to_string()))?;
    Ok(decrypted.to_vec())
}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let AppState::Processing = app.state {
            // Start processing in background
            let input = app.input.clone();
            let provider = app.provider.clone();
            let model = app.model.clone();
            let temperature = app.temperature;
            let max_tokens = app.max_tokens;
            let stream = app.stream;
            let config = app.config.clone();
            let args = app.args.clone();
            let tx_process = tx.clone();
            let tx_error = tx.clone();

            tokio::spawn(async move {
                if let Err(e) = process_query(input, provider, model, temperature, max_tokens, stream, config, args, tx_process).await {
                    let _ = tx_error.send(format!("Error: {:?}", e));
                }
            });

            app.state = AppState::Response;
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
                                // Clear messages
                                app.messages.clear();
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
                                app.messages.push(Message {
                                    sender: "You".to_string(),
                                    content: app.input.clone(),
                                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                                });
                                save_chat_history(&app).ok();
                                app.history_index = None;
                                app.state = AppState::Processing;
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
            app.messages.push(Message {
                sender: app.provider.name().to_string(),
                content: response,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            });
            save_chat_history(&app).ok();
            app.state = AppState::Input;
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
    stream: bool,
    config: Config,
    args: Args,
    tx: mpsc::UnboundedSender<String>,
) -> Result<()> {
    use crate::api::{ApiClient, Message, ChatRequest};

    let api_key = args.api_key.as_ref().or(config.api.api_key.as_ref()).cloned();
    let timeout = args.timeout.unwrap_or(config.api.timeout);

    let client = ApiClient::new(provider, api_key, timeout)?;

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
    let size = f.size();

    // Top layout: status and main
    let top_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(size);

    // Status bar
    let status_text = format!("Provider: {:?} | Model: {} | Temp: {:.1} | Max Tokens: {} | Stream: {}",
        app.provider, app.model, app.temperature, app.max_tokens.unwrap_or(0), if app.stream { "On" } else { "Off" });
    let status = Paragraph::new(status_text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(status, top_chunks[0]);

    // Main layout: sidebar and main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(top_chunks[1]);

    // Sidebar: History
    let history_items: Vec<ListItem> = app
        .messages
        .iter()
        .rev()
        .take(10)
        .map(|m| {
            let preview = if m.content.len() > 20 { format!("{}...", &m.content[..20]) } else { m.content.clone() };
            ListItem::new(format!("{}: {}", m.sender, preview))
        })
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

    let inner_area = right_chunks[1].inner(&Margin::new(1, 1));

    let mut lines = Vec::new();
    for message in &app.messages {
        let timestamp = std::time::SystemTime::now().duration_since(message.timestamp).unwrap().as_secs();
        let time_str = if timestamp < 60 {
            format!("{}s ago", timestamp)
        } else if timestamp < 3600 {
            format!("{}m ago", timestamp / 60)
        } else {
            format!("{}h ago", timestamp / 3600)
        };
        if message.sender == "You" {
            lines.push(Line::from(vec![
                Span::styled("You:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(&message.content, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(format!("({})", time_str), Style::default().fg(Color::Gray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("AI:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(&message.content, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(format!("({})", time_str), Style::default().fg(Color::Gray)),
            ]));
        }
        lines.push(Line::raw("")); // Empty line between messages
    }
        lines.push(Line::raw("")); // Empty line between messages
    }

    match app.state {
        AppState::Input => {
            if app.messages.is_empty() {
                let text = "Enter your prompt and press Enter...";
                let para = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Gray))
                    .wrap(Wrap { trim: true });
                f.render_widget(para, inner_area);
            } else {
                let para = Paragraph::new(lines.clone())
                    .wrap(Wrap { trim: true });
                f.render_widget(para, inner_area);
            }
        }
        AppState::Processing => {
            let text = format!("{} is thinking...", app.provider.name());
            let para = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: true });
            f.render_widget(para, inner_area);
        }
        AppState::Response => {
            let para = Paragraph::new(lines)
                .wrap(Wrap { trim: true });
            f.render_widget(para, inner_area);
        }
    }

    // Input area
    let input_block = Block::default().borders(Borders::ALL).title(" Input").border_style(Style::default().fg(Color::Blue));
    f.render_widget(input_block, right_chunks[2]);

    let input_area = right_chunks[2].inner(&Margin::new(1, 1));
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
        f.set_cursor(
            right_chunks[2].x + app.input.len() as u16 + 1,
            right_chunks[2].y + 1,
        );
    }
}