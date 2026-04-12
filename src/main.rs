mod api;
mod cli;
mod completion;
mod config;
mod error;
mod platform;
mod repl;
mod tui;

use api::{ApiClient, ChatRequest, Message, Provider};
use chrono::Utc;
use clap::Parser;
use cli::Args;
use colored::Colorize;
use config::Config;
use error::{EchomindError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, OpenOptions};
use std::io::{IsTerminal, Write};
use std::net::TcpStream;
use std::time::Duration;
use tokio::io::{self, AsyncReadExt};

const MAX_INPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    if let Some(shell) = args.generate_completion {
        completion::generate_completion(shell);
        return Ok(());
    }

    // Quick dispatch for config commands (no network needed)
    if args.init_config {
        return Config::init_default_config();
    }
    if args.show_config {
        let path = Config::config_path()?;
        println!("Config file path: {}", path.display());
        if path.exists() {
            println!("{}", fs::read_to_string(&path).unwrap_or_default());
        }
        return Ok(());
    }
    if args.list_providers {
        for p in &["chat", "chatanywhere", "openai", "claude", "ollama", "grok", "mistral", "cohere", "gemini", "custom:<url>"] {
            println!("  {}", p);
        }
        return Ok(());
    }

    if !check_internet() {
        return Err(EchomindError::NetworkError(
            "No internet connection. Check your network and try again.".into(),
        ));
    }

    let config = Config::load()?;

    let mut initial_messages: Vec<Message> = Vec::new();
    let mut system_prompt = args.system.clone();

    if let Some(ref preset_name) = args.preset {
        let preset = config.presets.get(preset_name).ok_or_else(|| {
            EchomindError::ConfigError(format!("Preset '{}' not found in config.", preset_name))
        })?;
        if let Some(ref sp) = preset.system_prompt {
            system_prompt = Some(sp.clone());
        }
        if let Some(ref msgs) = preset.messages {
            initial_messages.extend(msgs.clone());
        }
    }

    // TUI mode
    if args.tui {
        return run_tui(args, config).await;
    }

    // Interactive mode
    if args.interactive {
        return run_interactive(args, config, initial_messages, system_prompt).await;
    }

    // Batch mode
    if let Some(ref batch_file) = args.batch {
        return run_batch(batch_file, &args, &config, &initial_messages, &system_prompt).await;
    }

    // Compare mode
    if let Some(ref models_str) = args.compare {
        let input = read_input(&args).await?;
        return compare_models(&input, models_str, &args, &config, system_prompt).await;
    }

    // Single query
    let input = if args.clipboard {
        platform::clipboard::read_from_clipboard()?
    } else if std::io::stdin().is_terminal() {
        print_usage();
        return Ok(());
    } else {
        read_stdin().await?
    };

    run_single_query(&args, &config, &input, initial_messages, system_prompt).await
}

// ── TUI ───────────────────────────────────────────────────────────────

async fn run_tui(args: Args, config: Config) -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};

    let mut stdout = std::io::stdout();
    enable_raw_mode().ok();

    if execute!(stdout, EnterAlternateScreen, EnableMouseCapture).is_err() {
        disable_raw_mode().ok();
        return run_interactive(args, config, vec![], None).await;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let app = tui::TuiApp::new(config, args);
    let res = tui::run_tui(&mut terminal, app).await;

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture).ok();
    terminal.show_cursor().ok();

    if let Err(err) = res {
        eprintln!("{} TUI error: {:?}", "Error:".red().bold(), err);
    }
    Ok(())
}

// ── Interactive ───────────────────────────────────────────────────────

pub async fn run_interactive(
    args: Args,
    config: Config,
    initial_messages: Vec<Message>,
    system_prompt: Option<String>,
) -> Result<()> {
    let api_key = args.api_key.clone().or_else(|| config.api.api_key.clone());
    let timeout = args.timeout.unwrap_or(config.api.timeout);
    let provider = Provider::from_string(
        args.provider.as_deref().unwrap_or(&config.api.provider),
    )
    .unwrap_or(Provider::Chat);
    let client = ApiClient::new(provider, api_key, timeout)?;
    let mut repl = repl::Repl::new(
        client,
        config,
        args.temperature,
        args.max_tokens,
        args.model.clone(),
        args.stream,
        initial_messages,
        system_prompt,
    );
    repl.run().await
}

// ── Batch ─────────────────────────────────────────────────────────────

async fn run_batch(
    batch_file: &str,
    args: &Args,
    config: &Config,
    initial_messages: &[Message],
    system_prompt: &Option<String>,
) -> Result<()> {
    let contents = fs::read_to_string(batch_file)
        .map_err(|e| EchomindError::FileError(format!("Failed to read batch file: {}", e)))?;

    for (i, line) in contents.lines().enumerate() {
        let query = line.trim();
        if query.is_empty() || query.starts_with('#') {
            continue;
        }
        println!("{}", "─".repeat(80).bright_black());
        println!("{}", format!("Batch Query {}: {}", i + 1, query).cyan().bold());
        println!("{}", "─".repeat(80).bright_black());

        run_single_query(args, config, query, initial_messages.to_vec(), system_prompt.clone()).await?;
        println!();
    }
    Ok(())
}

// ── Single Query ──────────────────────────────────────────────────────

async fn run_single_query(
    args: &Args,
    config: &Config,
    input: &str,
    mut messages: Vec<Message>,
    system_prompt: Option<String>,
) -> Result<()> {
    let start = std::time::Instant::now();
    let (coder, output) = args.resolve_coder_and_output();

    let provider_str = args.provider.as_deref().unwrap_or(&config.api.provider);
    let mut provider = Provider::from_string(provider_str)?;
    let mut fallback_chain = config.api.fallback_providers.clone();
    let mut api_key = args.api_key.clone().or_else(|| config.api.api_key.clone());
    let timeout = args.timeout.unwrap_or(config.api.timeout);
    let model = args.model.as_deref().unwrap_or(&config.api.model);

    if args.audit_log {
        log_audit(provider_str, model, input)?;
    }

    // Create client with interactive key prompt fallback
    let mut client = match ApiClient::new(provider.clone(), api_key.clone(), timeout) {
        Ok(c) => c,
        Err(EchomindError::MissingApiKey(_)) if std::io::stdin().is_terminal() => {
            eprintln!("{} Missing API key for {}", "⚠".yellow(), provider_str);
            eprintln!("Paste your API key and press Enter:");
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).ok();
            let key = buf.trim().to_string();
            if !key.is_empty() {
                api_key = Some(key.clone());
                let mut cfg = config.clone();
                cfg.api.api_key = Some(key);
                cfg.save()?;
            }
            ApiClient::new(provider.clone(), api_key.clone(), timeout)?
        }
        Err(e) => return Err(e),
    };

    // Load history
    if let Some(ref hf) = args.history {
        messages.extend(load_history(hf)?);
    }

    // System prompt
    if let Some(sp) = system_prompt {
        messages.push(Message::new("system", sp));
    } else if coder {
        messages.push(Message::new(
            "system",
            "You are a code generator. Output only raw, runnable code. No explanations, no markdown fences.",
        ));
    }

    // User message
    let user_content = match &args.prompt {
        Some(prompt) => format!("{}\n\n{}", input.trim(), prompt),
        None => input.trim().to_string(),
    };
    messages.push(Message::new("user", &user_content));

    let request = ChatRequest {
        messages: messages.clone(),
        model: Some(model.to_string()),
        temperature: args.temperature.or(Some(config.defaults.temperature)),
        max_tokens: args.max_tokens.or(config.defaults.max_tokens),
        top_p: args.top_p.or(config.defaults.top_p),
        top_k: args.top_k.or(config.defaults.top_k),
        stream: if args.stream { Some(true) } else { None },
    };

    if args.verbose {
        eprintln!("{} {}", "Provider:".cyan(), provider_str);
        eprintln!("{} {}", "Model:".cyan(), model);
    }

    // Progress spinner
    let progress = if !args.stream && std::io::stderr().is_terminal() {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message("Thinking...");
        pb.enable_steady_tick(Duration::from_millis(80));
        Some(pb)
    } else {
        None
    };

    // Send with fallback
    let content = loop {
        let attempt = if args.stream {
            client
                .send_message_stream(request.clone(), |chunk| {
                    print!("{}", chunk);
                    std::io::stdout().flush().ok();
                })
                .await
        } else {
            client.send_message(request.clone()).await
        };

        match attempt {
            Ok(ok) => break ok,
            Err(e) => {
                if let Some(next) = fallback_chain.first().cloned() {
                    fallback_chain.remove(0);
                    provider = Provider::from_string(&next)?;
                    client = ApiClient::new(provider.clone(), api_key.clone(), timeout)?;
                } else {
                    return Err(e);
                }
            }
        }
    };

    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    // Process coder mode output
    let output_content = if coder { strip_code_fences(&content) } else { content.clone() };

    // Format
    let formatted = match &args.format {
        Some(fmt) => format_output(&output_content, fmt, provider_str, model)?,
        None => output_content,
    };

    // Output
    if let Some(ref outfile) = output {
        fs::write(outfile, &formatted).map_err(|e| EchomindError::FileError(e.to_string()))?;
        println!("{} {}", "✅ Saved to".green(), outfile);
    }
    if args.to_clipboard {
        platform::clipboard::copy_to_clipboard(&formatted)?;
        println!("{}", "✅ Copied to clipboard".green());
    }
    if output.is_none() {
        if args.stream {
            println!();
        } else {
            println!("{}", formatted);
        }
    }

    // Save history
    if let Some(ref hf) = args.history {
        save_history(hf, &user_content, &content, provider_str, model)?;
        if args.verbose {
            eprintln!("{}", "✅ Saved to history".green());
        }
    }

    if args.verbose {
        eprintln!("{} {:.2}s", "⏱️  Total time:".cyan(), start.elapsed().as_secs_f64());
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────

fn check_internet() -> bool {
    ["8.8.8.8:53", "1.1.1.1:53", "208.67.222.222:53"]
        .iter()
        .any(|ep| TcpStream::connect_timeout(&ep.parse().unwrap(), Duration::from_secs(2)).is_ok())
}

async fn read_input(args: &Args) -> Result<String> {
    if args.clipboard {
        platform::clipboard::read_from_clipboard()
    } else if std::io::stdin().is_terminal() {
        Err(EchomindError::InputError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No input provided. Use --clipboard or pipe input.",
        )))
    } else {
        read_stdin().await
    }
}

async fn read_stdin() -> Result<String> {
    let mut input = String::new();
    let mut buf = [0u8; 8192];
    let mut total = 0;
    loop {
        let n = io::stdin().read(&mut buf).await?;
        if n == 0 {
            break;
        }
        total += n;
        if total > MAX_INPUT_SIZE {
            return Err(EchomindError::InputError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Input exceeds {}MB limit", MAX_INPUT_SIZE / (1024 * 1024)),
            )));
        }
        input.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    if input.trim().is_empty() {
        return Err(EchomindError::InputError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No input provided",
        )));
    }
    Ok(input)
}

fn strip_code_fences(content: &str) -> String {
    let mut lines: Vec<&str> = content.lines().collect();
    if lines.first().is_some_and(|l| l.trim().starts_with("```")) {
        lines.remove(0);
    }
    if lines.last().is_some_and(|l| l.trim() == "```") {
        lines.pop();
    }
    lines.into_iter().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n")
}

fn format_output(content: &str, fmt: &str, provider: &str, model: &str) -> Result<String> {
    match fmt {
        "json" => {
            let out = serde_json::json!({
                "content": content,
                "provider": provider,
                "model": model,
                "timestamp": Utc::now().to_rfc3339()
            });
            Ok(serde_json::to_string_pretty(&out)?)
        }
        "text" => Ok(content.to_string()),
        _ if fmt.starts_with("template:") => {
            Ok(fmt[9..]
                .replace("{content}", content)
                .replace("{provider}", provider)
                .replace("{model}", model)
                .replace("{timestamp}", &Utc::now().to_rfc3339()))
        }
        _ => Err(EchomindError::Other(format!("Unknown format: {}", fmt))),
    }
}

fn log_audit(provider: &str, model: &str, input: &str) -> Result<()> {
    let dir = Config::config_path()?.parent().unwrap().join("audit");
    fs::create_dir_all(&dir).map_err(|e| EchomindError::FileError(e.to_string()))?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("audit.log"))
        .map_err(|e| EchomindError::FileError(e.to_string()))?;
    writeln!(
        f,
        "{}",
        serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "provider": provider,
            "model": model,
            "input_length": input.len(),
        })
    )
    .map_err(|e| EchomindError::FileError(e.to_string()))
}

fn load_history(path: &str) -> Result<Vec<Message>> {
    if !std::path::Path::new(path).exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)
        .map_err(|e| EchomindError::FileError(format!("Failed to read history: {}", e)))?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&data)
        .map_err(|e| EchomindError::ParseError(format!("Failed to parse history: {}", e)))?;
    Ok(entries
        .into_iter()
        .filter_map(|e| {
            let role = e.get("role")?.as_str()?.to_string();
            let content = e.get("content")?.as_str()?.to_string();
            Some(Message::new(role, content))
        })
        .collect())
}

fn save_history(path: &str, user_msg: &str, assistant_msg: &str, provider: &str, model: &str) -> Result<()> {
    let mut entries: Vec<serde_json::Value> = if std::path::Path::new(path).exists() {
        let data = fs::read_to_string(path).unwrap_or_else(|_| "[]".into());
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let now = Utc::now().to_rfc3339();
    entries.push(serde_json::json!({ "timestamp": now, "role": "user", "content": user_msg, "provider": provider, "model": model }));
    entries.push(serde_json::json!({ "timestamp": now, "role": "assistant", "content": assistant_msg, "provider": provider, "model": model }));

    fs::write(path, serde_json::to_string_pretty(&entries)?)
        .map_err(|e| EchomindError::FileError(format!("Failed to write history: {}", e)))
}

async fn compare_models(
    input: &str,
    models_str: &str,
    args: &Args,
    config: &Config,
    system_prompt: Option<String>,
) -> Result<()> {
    let models: Vec<&str> = models_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if models.is_empty() {
        return Err(EchomindError::Other("No models specified for comparison".into()));
    }

    println!("{}", "=== Multi-Model Comparison ===".cyan().bold());
    println!("{}: {}\n", "Input".yellow(), input);

    let mut tasks = Vec::with_capacity(models.len());
    for model_name in models {
        let input = input.to_string();
        let sp = system_prompt.clone();
        let args = args.clone();
        let config = config.clone();
        let model_name = model_name.to_string();

        tasks.push(tokio::spawn(async move {
            let (prov, actual_model) = infer_provider(&model_name, &args, &config);
            let provider = Provider::from_string(prov)?;
            let key = args.api_key.clone().or_else(|| config.api.api_key.clone());
            let client = ApiClient::new(provider, key, args.timeout.unwrap_or(config.api.timeout))?;

            let mut msgs = Vec::new();
            if let Some(s) = sp {
                msgs.push(Message::new("system", s));
            }
            msgs.push(Message::new("user", &input));

            let req = ChatRequest {
                messages: msgs,
                model: Some(actual_model.clone()),
                temperature: args.temperature.or(Some(config.defaults.temperature)),
                max_tokens: args.max_tokens.or(config.defaults.max_tokens),
                top_p: None, top_k: None, stream: None,
            };
            let result = client.send_message(req).await;
            Ok::<_, EchomindError>((actual_model, result))
        }));
    }

    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok((name, result))) => results.push((name, result)),
            Ok(Err(e)) => eprintln!("Task error: {}", e),
            Err(e) => eprintln!("Join error: {}", e),
        }
    }
    results.sort_by(|a, b| a.0.cmp(&b.0));

    for (name, result) in results {
        println!("{} {}", "Model:".green().bold(), name);
        println!("{}", "─".repeat(80).bright_black());
        match result {
            Ok(r) => println!("{}", r),
            Err(e) => println!("{} {}", "Error:".red(), e),
        }
        println!("{}\n", "─".repeat(80).bright_black());
    }
    Ok(())
}

fn infer_provider<'a>(model: &str, args: &'a Args, config: &'a Config) -> (&'a str, String) {
    if model.starts_with("gpt") {
        return ("openai", model.to_string());
    }
    if model.starts_with("claude") {
        return ("claude", model.to_string());
    }
    if model.starts_with("gemini") {
        return ("gemini", model.to_string());
    }
    if let Some(i) = model.find('/') {
        let (prov, m) = model.split_at(i);
        return (
            // Leak is safe here — these are short-lived CLI strings
            Box::leak(prov.to_string().into_boxed_str()),
            m[1..].to_string(),
        );
    }
    (
        args.provider.as_deref().unwrap_or(&config.api.provider),
        model.to_string(),
    )
}

fn print_usage() {
    println!("{}", "Echomind - AI Chat CLI Tool".cyan().bold());
    println!("\n{}", "Usage:".yellow().bold());
    println!("  echo 'message' | echomind [OPTIONS]");
    println!("  echomind --interactive");
    println!("  echomind --tui");
    println!("\n{}", "Options:".yellow().bold());
    println!("  -c, --coder           Coder mode (clean code output)");
    println!("  -o, --output <FILE>   Save to file");
    println!("  -p, --provider <NAME> Provider (chat, openai, claude, gemini, ollama)");
    println!("  -m, --model <MODEL>   Model name");
    println!("  -i, --interactive     Interactive REPL");
    println!("  --tui                 TUI chat interface");
    println!("  --stream              Stream responses");
    println!("  --clipboard           Read from clipboard");
    println!("  -h, --help            Full help");
    println!("\nRun: echomind --help for all options");
}
