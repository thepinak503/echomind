use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "echomind")]
#[command(version)]
#[command(about = "Send piped input to AI chat API and print response")]
#[command(
    long_about = "A lightweight, fast command-line tool that pipes input to an AI chat API and outputs the response.

Examples:
  echo 'Hello, how are you?' | echomind
  cat file.txt | echomind
  echo 'write a Python function' | echomind --coder --output code.py
  echomind --interactive
  echomind --init-config
  echo 'explain quantum computing' | echomind --provider openai --model gpt-4"
)]
pub struct Args {
    /// Enable coder mode (generates clean code without explanations)
    #[arg(short = 'c', long)]
    pub coder: bool,

    /// Save response to a file instead of printing to stdout
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Combined --coder --output (shorthand)
    #[arg(long)]
    pub co: Option<String>,

    /// API provider to use (chat, chatanywhere, openai, claude, ollama, grok, mistral, cohere, or custom URL)
    #[arg(short = 'p', long)]
    pub provider: Option<String>,

    /// Model to use (e.g., gpt-3.5-turbo, gpt-4, claude-3-opus)
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Temperature for response randomness (0.0-2.0)
    #[arg(short = 't', long)]
    pub temperature: Option<f32>,

    /// Maximum tokens in response
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Custom system prompt
    #[arg(short = 's', long)]
    pub system: Option<String>,

    /// Enable streaming mode (display response as it arrives)
    #[arg(long)]
    pub stream: bool,

    /// Interactive REPL mode for multi-turn conversations
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// API key for the provider (can also be set via ECHOMIND_API_KEY env var)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Request timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Enable verbose output for debugging
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Initialize default configuration file
    #[arg(long)]
    pub init_config: bool,

    /// Show configuration file path
    #[arg(long)]
    pub show_config: bool,

    /// Read input from clipboard instead of stdin
    #[arg(long)]
    pub clipboard: bool,

    /// Save response to clipboard
    #[arg(long)]
    pub to_clipboard: bool,

    /// Conversation history file for persistent context
    #[arg(long)]
    pub history: Option<String>,

    /// Compare responses from multiple models (comma-separated)
    #[arg(long)]
    pub compare: Option<String>,

    /// Output format: text, json, or template:<template>
    #[arg(long)]
    pub format: Option<String>,

    /// Image file to include with the request (for vision models)
    #[arg(long)]
    pub image: Option<String>,

    /// Optional prompt to append to input (useful when piping)
    #[arg(value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Use a predefined conversation preset from the config file
    #[arg(long)]
    pub preset: Option<String>,

    /// Process multiple queries from a file, one per line
    #[arg(long)]
    pub batch: Option<String>,
}

impl Args {
    pub fn resolve_coder_and_output(&self) -> (bool, Option<String>) {
        if let Some(co_file) = &self.co {
            (true, Some(co_file.clone()))
        } else {
            (self.coder, self.output.clone())
        }
    }
}
