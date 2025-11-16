use crate::api::{ApiClient, ChatRequest, Message, Provider};
use crate::cli::Args;
use crate::config::Config;
use crate::error::{EchomindError, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap,
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
            let tx = tx.clone();

            task::spawn(async move {
                if let Err(e) = process_query(input, provider, model, temperature, max_tokens, top_p, top_k, stream, config, args, tx).await {
                    let _ = tx.send(format!("Error: {:?}", e));
                }
            });

            app.next();
        }

        if let Ok(event) = event::read() {
            match event {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        if let AppState::Input = app.state {
                            if !app.input.is_empty() {
                                app.history.push(app.input.clone());
                                app.next();
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if let AppState::Input = app.state {
                            app.input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if let AppState::Input = app.state {
                            app.input.pop();
                        }
                    }
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                },
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

    let mut client = ApiClient::new(provider, api_key, timeout)?;

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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs/Settings
            Constraint::Min(1),    // Main area
            Constraint::Length(3), // Input
        ])
        .split(size);

    // Settings bar
    let settings = vec![
        format!("Provider: {}", app.provider.name()),
        format!("Model: {}", app.model),
        format!("Temp: {:.1}", app.temperature),
        format!("Max Tokens: {}", app.max_tokens.unwrap_or(2000)),
        format!("Top P: {:.2}", app.top_p.unwrap_or(1.0)),
        format!("Top K: {}", app.top_k.unwrap_or(0)),
        format!("Stream: {}", app.stream),
    ];
    let settings_text = settings.join(" | ");
    let settings_para = Paragraph::new(settings_text)
        .block(Block::default().borders(Borders::ALL).title("Settings"));
    f.render_widget(settings_para, chunks[0]);

    // Main area
    let main_block = Block::default().borders(Borders::ALL).title("Response");
    f.render_widget(main_block, chunks[1]);

    let inner_area = chunks[1].inner(&Margin { vertical: 1, horizontal: 1 });

    match app.state {
        AppState::Input => {
            let text = "Enter your prompt and press Enter...";
            let para = Paragraph::new(text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(para, inner_area);
        }
        AppState::Processing => {
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title("Processing"))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(50);
            f.render_widget(gauge, inner_area);
        }
        AppState::Response => {
            let para = Paragraph::new(app.response.as_str())
                .wrap(Wrap { trim: true });
            f.render_widget(para, inner_area);
        }
    }

    // Input area
    let input_block = Block::default().borders(Borders::ALL).title("Input");
    f.render_widget(input_block, chunks[2]);

    let input_area = chunks[2].inner(&Margin { vertical: 1, horizontal: 1 });
    let input_para = Paragraph::new(app.input.as_str());
    f.render_widget(input_para, input_area);

    // Set cursor
    if let AppState::Input = app.state {
        f.set_cursor(
            chunks[2].x + app.input.len() as u16 + 1,
            chunks[2].y + 1,
        );
    }
}