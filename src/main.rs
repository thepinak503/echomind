mod api;
mod cli;
mod config;
mod error;
mod repl;

use api::{ApiClient, ChatRequest, Message, Provider};
use cli::Args;
use clap::Parser;
use colored::Colorize;
use config::Config;
use error::{EchomindError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::IsTerminal;
use tokio::io::{self, AsyncReadExt};
use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug)]
struct HistoryEntry {
    timestamp: DateTime<Utc>,
    role: String,
    content: String,
    provider: Option<String>,
    model: Option<String>,
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

    // Handle special flags first
    if args.init_config {
        Config::init_default_config()?;
        return Ok(());
    }

    if args.show_config {
        let config_path = Config::config_path()?;
        println!("Configuration file: {}", config_path.display());
        if config_path.exists() {
            println!("\nCurrent configuration:");
            let contents = fs::read_to_string(&config_path)
                .map_err(|e| EchomindError::FileError(e.to_string()))?;
            println!("{}", contents);
        } else {
            println!("{}", "Configuration file does not exist. Use --init-config to create it.".yellow());
        }
        return Ok(());
    }

    // Load configuration
    let config = Config::load()?;

    // Check if we're in interactive mode
    if args.interactive {
        return run_interactive(args, config).await;
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

        return compare_models(&input, models_str, &args, &config).await;
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
        println!("  -p, --provider <NAME>    API provider (chat, chatanywhere, openai, claude, ollama)");
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

    run_single_query(args, config, input).await
}

async fn run_single_query(args: Args, config: Config, input: String) -> Result<()> {
    let (coder, output) = args.resolve_coder_and_output();

    // Determine provider
    let provider_str = args.provider.as_ref()
        .unwrap_or(&config.api.provider);
    let provider = Provider::from_string(provider_str)?;

    // Get API key
    let api_key = args.api_key.or(config.api.api_key.clone());

    // Get timeout
    let timeout = args.timeout.unwrap_or(config.api.timeout);

    // Get model
    let model = args.model.as_ref().unwrap_or(&config.api.model).clone();

    // Create API client
    let client = ApiClient::new(provider, api_key, timeout)?;

    // Build messages
    let mut messages = Vec::new();

    // Load history if specified
    if let Some(history_file) = &args.history {
        let history_messages = load_history(history_file)?;
        messages.extend(history_messages);
    }

    // Add system message if in coder mode or custom system prompt
    if let Some(system_prompt) = args.system {
        messages.push(Message {
            role: "system".to_string(),
            content: system_prompt,
        });
    } else if coder {
        messages.push(Message {
            role: "system".to_string(),
            content: "You are a code generator. Always and only output raw, runnable code with no explanations, comments, markdown fences, or prose. Do not include code block syntax like triple backticks.".to_string(),
        });
    }

    // Add user message (combine input with optional prompt)
    let user_content = if let Some(prompt) = &args.prompt {
        format!("{}\n\n{}", input.trim(), prompt)
    } else {
        input.trim().to_string()
    };

    let user_message = Message {
        role: "user".to_string(),
        content: user_content,
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

    // Send request
    let content = if args.stream {
        client.send_message_stream(request, |chunk| {
            print!("{}", chunk);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }).await?
    } else {
        let resp = client.send_message(request).await?;
        resp
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

        lines.into_iter()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        content.clone()
    };

    // Save to file if specified
    if let Some(outfile) = &output {
        fs::write(outfile, &output_content)
            .map_err(|e| EchomindError::FileError(e.to_string()))?;
        println!("{} {}", "✅ Saved to".green(), outfile);
    }

    // Save to clipboard if specified
    if args.to_clipboard {
        write_to_clipboard(&output_content)?;
        println!("{}", "✅ Copied to clipboard".green());
    }

    // Display output if not saved to file
    if output.is_none() {
        if !args.stream {
            println!("{}", output_content);
        } else {
            println!(); // Add newline after streaming
        }
    }

    // Save to history if specified
    if let Some(history_file) = &args.history {
        let mut history_messages = vec![user_message];
        history_messages.push(Message {
            role: "assistant".to_string(),
            content,
        });
        save_history(history_file, &history_messages, provider_str, &model)?;
        if args.verbose {
            eprintln!("{}", "✅ Saved to history".green());
        }
    }

    Ok(())
}

async fn run_interactive(args: Args, config: Config) -> Result<()> {
    // Determine provider
    let provider_str = args.provider.as_ref()
        .unwrap_or(&config.api.provider);
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
    );

    repl.run().await
}

// Helper function to read from clipboard
fn read_from_clipboard() -> Result<String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| EchomindError::Other(format!("Failed to access clipboard: {}", e)))?;

    clipboard.get_text()
        .map_err(|e| EchomindError::Other(format!("Failed to read from clipboard: {}", e)))
}

// Helper function to write to clipboard
fn write_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| EchomindError::Other(format!("Failed to access clipboard: {}", e)))?;

    clipboard.set_text(text)
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

    Ok(entries.into_iter().map(|e| Message {
        role: e.role,
        content: e.content,
    }).collect())
}

// Save conversation history
fn save_history(history_file: &str, messages: &[Message], provider: &str, model: &str) -> Result<()> {
    let entries: Vec<HistoryEntry> = messages.iter().map(|msg| HistoryEntry {
        timestamp: Utc::now(),
        role: msg.role.clone(),
        content: msg.content.clone(),
        provider: Some(provider.to_string()),
        model: Some(model.to_string()),
    }).collect();

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| EchomindError::ParseError(format!("Failed to serialize history: {}", e)))?;

    fs::write(history_file, json)
        .map_err(|e| EchomindError::FileError(format!("Failed to write history: {}", e)))?;

    Ok(())
}

// Compare responses from multiple models
async fn compare_models(
    input: &str,
    models_str: &str,
    args: &Args,
    config: &Config,
) -> Result<()> {
    let models: Vec<&str> = models_str.split(',').map(|s| s.trim()).collect();

    if models.is_empty() {
        return Err(EchomindError::Other("No models specified for comparison".to_string()));
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
            (args.provider.as_deref().unwrap_or(&config.api.provider), model_name)
        };

        let provider = Provider::from_string(provider_name)?;
        let api_key = args.api_key.as_ref().or(config.api.api_key.as_ref()).cloned();
        let timeout = args.timeout.unwrap_or(config.api.timeout);

        let client = ApiClient::new(provider, api_key, timeout)?;

        let messages = vec![Message {
            role: "user".to_string(),
            content: input.to_string(),
        }];

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
