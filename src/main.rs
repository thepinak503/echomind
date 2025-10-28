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

    // Check if stdin is a terminal and no input is being piped
    if std::io::stdin().is_terminal() {
        // Show help when running echomind without input
        println!("{}", "Echomind - AI Chat CLI Tool".cyan().bold());
        println!("\n{}", "Usage:".yellow().bold());
        println!("  echo 'your message' | echomind [OPTIONS]");
        println!("  cat file.txt | echomind [OPTIONS]");
        println!("  echomind --interactive");
        println!("\n{}", "Common Options:".yellow().bold());
        println!("  -c, --coder              Enable coder mode (clean code output)");
        println!("  -o, --output <FILE>      Save response to file");
        println!("  -p, --provider <NAME>    API provider (chat, chatanywhere, openai, claude, ollama)");
        println!("  -m, --model <MODEL>      Model to use");
        println!("  -i, --interactive        Interactive REPL mode");
        println!("  -t, --temperature <NUM>  Temperature (0.0-2.0)");
        println!("  --stream                 Stream response as it arrives");
        println!("  --init-config            Create default config file");
        println!("  -h, --help               Show detailed help");
        println!("\n{}", "Examples:".yellow().bold());
        println!("  echo 'Hello AI' | echomind");
        println!("  echo 'write a Python function' | echomind -c -o code.py");
        println!("  echomind --interactive --stream");
        println!("  echo 'explain Docker' | echomind --provider openai --model gpt-4");
        println!("\nFor more information, run: echomind --help");
        return Ok(());
    }

    // Read input from stdin
    let mut input = String::new();
    match io::stdin().read_to_string(&mut input).await {
        Ok(_) => {
            if input.trim().is_empty() {
                // No input provided, show help
                println!("{}", "Echomind - AI Chat CLI Tool".cyan().bold());
                println!("\n{}", "Usage:".yellow().bold());
                println!("  echo 'your message' | echomind [OPTIONS]");
                println!("\n{}", "Error:".red().bold());
                println!("  No input provided. Pipe text to echomind or use --interactive mode.");
                println!("\nFor help, run: echomind --help");
                return Err(EchomindError::InputError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "No input provided",
                )));
            }
        }
        Err(e) => return Err(EchomindError::InputError(e)),
    }

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

    // Create API client
    let client = ApiClient::new(provider, api_key, timeout)?;

    // Build messages
    let mut messages = Vec::new();

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

    // Add user message
    messages.push(Message {
        role: "user".to_string(),
        content: input.trim().to_string(),
    });

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

    // Handle output
    if let Some(outfile) = output {
        let cleaned = if coder {
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

        fs::write(&outfile, cleaned)
            .map_err(|e| EchomindError::FileError(e.to_string()))?;
        println!("{} {}", "✅ Saved to".green(), outfile);
    } else if !args.stream {
        println!("{}", content);
    } else {
        println!(); // Add newline after streaming
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
