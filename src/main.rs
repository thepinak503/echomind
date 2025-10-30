mod api;
mod cli;
mod config;
mod error;
mod repl;

use api::{ApiClient, ChatRequest, Message, Provider, ContentPart, ImageUrl};
use arboard::Clipboard;
use chrono::{DateTime, Utc};
use clap::Parser;
use cli::Args;
use colored::Colorize;
use config::Config;
use error::{EchomindError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::IsTerminal;
use tokio::io::{self, AsyncReadExt};
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize, Debug)]
struct HistoryEntry {
    timestamp: DateTime<Utc>,
    role: String,
    content: String,
    provider: Option<String>,
    model: Option<String>,
    has_image: bool,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();

    if args.init_config {
        return Config::init_default_config();
    }

    if args.show_config {
        let config_path = Config::config_path()?;
        println!("Config file path: {}", config_path.display());
        return Ok(());
    }

    let config = Config::load()?;

    let mut initial_messages: Vec<Message> = Vec::new();
    let mut system_prompt: Option<String> = args.system.clone();

    if let Some(preset_name) = args.preset.clone() {
        if let Some(preset) = config.presets.get(&preset_name) {
            if let Some(p_system_prompt) = &preset.system_prompt {
                system_prompt = Some(p_system_prompt.clone());
            }
            if let Some(p_messages) = &preset.messages {
                initial_messages.extend(p_messages.clone());
            }
        } else {
            return Err(EchomindError::ConfigError(format!("Preset '{}' not found in config.", preset_name)));
        }
    }

    // Check if we're in interactive mode
    if args.interactive {
        return run_interactive(args, config, initial_messages, system_prompt).await;
    }

    if let Some(batch_file) = &args.batch {
        return run_batch_queries(batch_file, args.clone(), config, initial_messages, system_prompt).await;
    }

    // Check for model comparison mode
    if let Some(models_str) = &args.compare {
        // Read input from clipboard or stdin
        let input = if args.clipboard {
            read_from_clipboard()?
        } else if std::io::stdin().is_terminal() {
            return Err(EchomindError::InputError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No input provided for comparison. Use --clipboard or pipe input.",
            )));
        } else {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).await?;
            input
        };

        return compare_models(&input, models_str, &args, &config, system_prompt).await;
    }

    // Read input: from clipboard, stdin, or show help
    let input = if args.clipboard {
        read_from_clipboard()?
    } else if std::io::stdin().is_terminal() {
        // Show help when running echomind without input
        println!("{}", "Echomind - AI Chat CLI Tool".cyan().bold());
        println!("\n{}", "Usage:".yellow().bold());
        println!("  echo 'your message' | echomind [OPTIONS]");
        println!("  cat file.txt | echomind [OPTIONS]");
        println!("  echomind --interactive");
        println!("  echomind --clipboard  # Read from clipboard");
        println!("\n{}", "Common Options:".yellow().bold());
        println!("  -c, --coder              Enable coder mode (clean code output)");
        println!("  -o, --output <FILE>      Save response to file");
        println!("  --clipboard              Read input from clipboard");
        println!("  --to-clipboard           Save response to clipboard");
        println!("  --history <FILE>         Conversation history file");
        println!("  --compare <MODELS>       Compare multiple models (comma-separated)");
        println!("  --format <FORMAT>        Output format (text, json, template:<template>)");
        println!(
            "  -p, --provider <NAME>    API provider (chat, chatanywhere, openai, claude, ollama, grok, mistral, cohere)"
        );
        println!("  -m, --model <MODEL>      Model to use");
        println!("  -i, --interactive        Interactive REPL mode");
        println!("  -t, --temperature <NUM>  Temperature (0.0-2.0)");
        println!("  --stream                 Stream response as it arrives");
        println!("  --init-config            Create default config file");
        println!("  -h, --help               Show detailed help");
        println!("\n{}", "Examples:".yellow().bold());
        println!("  echo 'Hello AI' | echomind");
        println!("  ls | echomind \"Explain these files\"");
        println!("  pbpaste | echomind  # or use --clipboard");
        println!("  echo 'code task' | echomind --compare gpt-4,claude-3-opus");
        println!("  git diff | echomind \"Summarize changes\"");
        println!("\nFor more information, run: echomind --help");
        return Ok(());
    } else {
        let mut input = String::new();
        match io::stdin().read_to_string(&mut input).await {
            Ok(_) => {
                if input.trim().is_empty() {
                    return Err(EchomindError::InputError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "No input provided",
                    )));
                }
                input
            }
            Err(e) => return Err(EchomindError::InputError(e)),
        }
    };

    run_single_query(args, config, input, initial_messages, system_prompt).await
}

async fn run_batch_queries(
    batch_file: &str,
    args: Args,
    config: Config,
    initial_messages: Vec<Message>,
    system_prompt: Option<String>,
) -> Result<()> {
    let contents = fs::read_to_string(batch_file)
        .map_err(|e| EchomindError::FileError(format!("Failed to read batch file: {}", e)))?;

    for (i, line) in contents.lines().enumerate() {
        let query = line.trim();
        if query.is_empty() || query.starts_with("#") {
            continue; // Skip empty lines and comments
        }

        println!("{}\n{}", "─".repeat(80).bright_black(), format!("Batch Query {}: {}", i + 1, query).cyan().bold());
        println!("{}", "─".repeat(80).bright_black());

        // Clone args and config for each query to avoid ownership issues
        run_single_query(
            args.clone(),
            config.clone(),
            query.to_string(),
            initial_messages.clone(),
            system_prompt.clone(),
        ).await?;

        println!(); // Add a newline for separation between responses
    }

    Ok(())
}

async fn run_single_query(args: Args, config: Config, input: String, messages: Vec<Message>, system_prompt: Option<String>) -> Result<()> {
    let (coder, output) = args.resolve_coder_and_output();

    // Determine provider and fallback chain
    let provider_str = args.provider.as_ref().unwrap_or(&config.api.provider);
    let mut provider = Provider::from_string(provider_str)?;
    let mut fallback_chain: Vec<String> = config.api.fallback_providers.clone();

    // Get API key
    let mut working_config = config.clone();
    let mut api_key = args.api_key.or(working_config.api.api_key.clone());

    // Get timeout
    let timeout = args.timeout.unwrap_or(config.api.timeout);

    // Get model
    let model = args.model.as_ref().unwrap_or(&config.api.model).clone();

    // Create API client (with key prompt/save on demand)
    let mut client = match ApiClient::new(provider.clone(), api_key.clone(), timeout) {
        Ok(c) => c,
        Err(EchomindError::MissingApiKey(_)) => {
            // Try to guide user to get an API key (Gemini case) and save it
            if std::io::stdin().is_terminal() {
                eprintln!(
                    "{} {}",
                    "Missing API key for provider".yellow(),
                    provider_str
                );
                eprintln!("Open Google AI Studio to create a Gemini key: https://aistudio.google.com/app/api-keys");
                eprintln!("Paste the API key here and press Enter (leave blank to skip):");
                let mut buf = String::new();
                std::io::stdin().read_line(&mut buf).ok();
                let entered = buf.trim();
                if !entered.is_empty() {
                    working_config.api.api_key = Some(entered.to_string());
                    working_config.save()?;
                    api_key = Some(entered.to_string());
                }
            }
            ApiClient::new(provider.clone(), api_key.clone(), timeout)?
        }
        Err(e) => return Err(e),
    };

    // Build messages
    let mut messages = messages;

    // Load history if specified
    if let Some(history_file) = &args.history {
        let history_messages = load_history(history_file)?;
        messages.extend(history_messages);
    }

    // Add system message if in coder mode or custom system prompt
    if let Some(s_prompt) = system_prompt {
        messages.push(Message::text("system".to_string(), s_prompt));
    } else if coder {
        messages.push(Message::text(
            "system".to_string(),
            "You are a code generator. Always and only output raw, runnable code with no explanations, comments, markdown fences, or prose. Do not include code block syntax like triple backticks.".to_string(),
        ));
    }

    // Add user message (combine input with optional prompt)
    let user_content = if let Some(prompt) = &args.prompt {
        format!("{}\n\n{}", input.trim(), prompt)
    } else {
        input.trim().to_string()
    };

    let user_message = if let Some(image_path) = &args.image {
        // Load image and create multimodal message
        let image_data = load_image_as_base64(image_path)?;
        let parts = vec![
            ContentPart::Text { text: user_content },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: format!("data:image/jpeg;base64,{}", image_data),
                },
            },
        ];
        Message::multimodal("user".to_string(), parts)
    } else {
        Message::text("user".to_string(), user_content)
    };
    messages.push(user_message.clone());

    // Build request
    let request = ChatRequest {
        messages,
        model: args.model.or(Some(config.api.model.clone())),
        temperature: args.temperature.or(Some(config.defaults.temperature)),
        max_tokens: args.max_tokens.or(config.defaults.max_tokens),
        stream: if args.stream { Some(true) } else { None },
    };

    if args.verbose {
        eprintln!("{} {:?}", "Provider:".cyan(), provider_str);
        eprintln!("{} {:?}", "Request:".cyan(), request);
    }

    // Show progress indicator
    let progress = if !args.stream && std::io::stderr().is_terminal() {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message("Thinking...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // Send request with fallback chain
    let content = loop {
        let attempt = if args.stream {
            client
                .send_message_stream(request.clone(), |chunk| {
                    print!("{}", chunk);
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                })
                .await
        } else {
            client.send_message(request.clone()).await
        };

        match attempt {
            Ok(ok) => break ok,
            Err(e) => {
                if let Some(next_provider_str) = fallback_chain.first().cloned() {
                    // Switch provider and retry
                    fallback_chain.remove(0);
                    provider = Provider::from_string(&next_provider_str)?;
                    client = ApiClient::new(provider.clone(), api_key.clone(), timeout)?;
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    };

    // Clear progress indicator
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    // Process output content
    let output_content = if coder {
        // Filter empty lines and remove markdown code fences
        let mut lines: Vec<&str> = content.lines().collect();

        // Remove markdown code fences
        if lines.first().map_or(false, |l| l.trim().starts_with("```")) {
            lines.remove(0);
        }
        if lines.last().map_or(false, |l| l.trim().starts_with("```")) {
            lines.pop();
        }

        lines
            .into_iter()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        content.clone()
    };

    // Format output if specified
    let formatted_output = if let Some(format_str) = &args.format {
        format_output(&output_content, format_str, provider_str, &model)?
    } else {
        output_content
    };

    // Save to file if specified
    if let Some(outfile) = &output {
        fs::write(outfile, &formatted_output).map_err(|e| EchomindError::FileError(e.to_string()))?;
        println!("{} {}", "✅ Saved to".green(), outfile);
    }

    // Save to clipboard if specified
    if args.to_clipboard {
        write_to_clipboard(&formatted_output)?;
        println!("{}", "✅ Copied to clipboard".green());
    }

    // Display output if not saved to file
    if output.is_none() {
        if !args.stream {
            println!("{}", formatted_output);
        } else {
            println!(); // Add newline after streaming
        }
    }

    // Save to history if specified
    if let Some(history_file) = &args.history {
        let mut history_messages = vec![user_message];
        history_messages.push(Message::text("assistant".to_string(), content));
        save_history(history_file, &history_messages, provider_str, &model)?;
        if args.verbose {
            eprintln!("{}", "✅ Saved to history".green());
        }
    }

    Ok(())
}

async fn run_interactive(args: Args, config: Config, initial_messages: Vec<Message>, system_prompt: Option<String>) -> Result<()> {
    // Determine provider
    let provider_str = args.provider.as_ref().unwrap_or(&config.api.provider);
    let provider = Provider::from_string(provider_str)?;

    // Get API key
    let api_key = args.api_key.or(config.api.api_key.clone());

    // Get timeout
    let timeout = args.timeout.unwrap_or(config.api.timeout);

    // Create API client
    let client = ApiClient::new(provider, api_key, timeout)?;

    // Create and run REPL
    let mut repl = repl::Repl::new(
        client,
        config.clone(),
        args.temperature,
        args.max_tokens,
        args.model,
        args.stream,
        initial_messages,
        system_prompt,
    );

    repl.run().await
}

// Helper function to read from clipboard
fn read_from_clipboard() -> Result<String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| EchomindError::Other(format!("Failed to access clipboard: {}", e)))?;

    clipboard
        .get_text()
        .map_err(|e| EchomindError::Other(format!("Failed to read from clipboard: {}", e)))
}

// Helper function to write to clipboard
fn write_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| EchomindError::Other(format!("Failed to access clipboard: {}", e)))?;

    clipboard
        .set_text(text)
        .map_err(|e| EchomindError::Other(format!("Failed to write to clipboard: {}", e)))
}

// Load conversation history
fn load_history(history_file: &str) -> Result<Vec<Message>> {
    if !std::path::Path::new(history_file).exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(history_file)
        .map_err(|e| EchomindError::FileError(format!("Failed to read history: {}", e)))?;

    let entries: Vec<HistoryEntry> = serde_json::from_str(&contents)
        .map_err(|e| EchomindError::ParseError(format!("Failed to parse history: {}", e)))?;

    Ok(entries
        .into_iter()
        .map(|e| Message::text(e.role, e.content))
        .collect())
}

// Save conversation history
fn save_history(
    history_file: &str,
    messages: &[Message],
    provider: &str,
    model: &str,
) -> Result<()> {
    let entries: Vec<HistoryEntry> = messages
        .iter()
        .map(|msg| HistoryEntry {
            timestamp: Utc::now(),
            role: msg.role.clone(),
            content: msg.get_text().unwrap_or("").to_string(),
            provider: Some(provider.to_string()),
            model: Some(model.to_string()),
            has_image: matches!(msg.content, api::MessageContent::MultiModal(_)),
        })
        .collect();

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| EchomindError::ParseError(format!("Failed to serialize history: {}", e)))?;

    fs::write(history_file, json)
        .map_err(|e| EchomindError::FileError(format!("Failed to write history: {}", e)))?;

    Ok(())
}

// Load image file and encode as base64
fn load_image_as_base64(path: &str) -> Result<String> {
    let data = fs::read(path).map_err(|e| EchomindError::FileError(format!("Failed to read image: {}", e)))?;
    Ok(general_purpose::STANDARD.encode(&data))
}

// Format output based on format specification
fn format_output(content: &str, format_str: &str, provider: &str, model: &str) -> Result<String> {
    match format_str {
        "json" => {
            let output = serde_json::json!({
                "content": content,
                "provider": provider,
                "model": model,
                "timestamp": Utc::now().to_rfc3339()
            });
            Ok(serde_json::to_string_pretty(&output)?)
        }
        "text" => Ok(content.to_string()),
        _ if format_str.starts_with("template:") => {
            let template = &format_str[9..]; // Remove "template:" prefix
            let formatted = template
                .replace("{content}", content)
                .replace("{provider}", provider)
                .replace("{model}", model)
                .replace("{timestamp}", &Utc::now().to_rfc3339());
            Ok(formatted)
        }
        _ => Err(EchomindError::Other(format!("Unknown format: {}", format_str))),
    }
}

// Compare responses from multiple models
async fn compare_models(input: &str, models_str: &str, args: &Args, config: &Config, system_prompt: Option<String>) -> Result<()> {
    let models: Vec<&str> = models_str.split(',').map(|s| s.trim()).collect();

    if models.is_empty() {
        return Err(EchomindError::Other(
            "No models specified for comparison".to_string(),
        ));
    }

    println!("{}", "=== Multi-Model Comparison ===".cyan().bold());
    println!("{}: {}\n", "Input".yellow(), input);

    for model_name in models {
        println!("{} {}", "Model:".green().bold(), model_name);
        println!("{}", "─".repeat(80).bright_black());

        // Determine provider from model name or use default
        let (provider_name, actual_model) = if model_name.starts_with("gpt") {
            ("openai", model_name)
        } else if model_name.starts_with("claude") {
            ("claude", model_name)
        } else if model_name.contains('/') {
            // Assume format like "ollama/llama2"
            let parts: Vec<&str> = model_name.split('/').collect();
            (parts[0], *parts.get(1).unwrap_or(&model_name))
        } else {
            (
                args.provider.as_deref().unwrap_or(&config.api.provider),
                model_name,
            )
        };

        let provider = Provider::from_string(provider_name)?;
        let api_key = args
            .api_key
            .as_ref()
            .or(config.api.api_key.as_ref())
            .cloned();
        let timeout = args.timeout.unwrap_or(config.api.timeout);

        let client = ApiClient::new(provider, api_key, timeout)?;

        let mut messages = Vec::new();
        if let Some(s_prompt) = system_prompt.clone() {
            messages.push(Message::text("system".to_string(), s_prompt));
        }
        messages.push(Message::text("user".to_string(), input.to_string()));

        let request = ChatRequest {
            messages,
            model: Some(actual_model.to_string()),
            temperature: args.temperature.or(Some(config.defaults.temperature)),
            max_tokens: args.max_tokens.or(config.defaults.max_tokens),
            stream: None,
        };

        match client.send_message(request).await {
            Ok(response) => {
                println!("{}", response);
            }
            Err(e) => {
                println!("{} {}", "Error:".red(), e);
            }
        }

        println!("{}\n", "─".repeat(80).bright_black());
    }

    Ok(())
}
