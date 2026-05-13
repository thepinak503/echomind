use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "echomind", version, about = "Send piped input to AI chat API and print response")]
#[command(long_about = "A powerful, cross-platform AI CLI tool for integrating AI into your workflow.

Examples:
  echo 'Hello' | echomind
  cat file.txt | echomind 'Explain this'
  echomind --interactive
  echomind --tui
  echo 'code task' | echomind --coder --output code.py")]
pub struct Args {
    /// Enable coder mode (generates clean code without explanations)
    #[arg(short = 'c', long)]
    pub coder: bool,

    /// Save response to a file
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Combined --coder --output (shorthand)
    #[arg(long)]
    pub co: Option<String>,

    /// API provider (chat, openai, claude, gemini, ollama, grok, mistral, cohere)
    #[arg(short = 'p', long)]
    pub provider: Option<String>,

    /// Model to use (e.g., gpt-4, claude-3-opus, gemini-pro)
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Temperature for response randomness (0.0-2.0)
    #[arg(short = 't', long)]
    pub temperature: Option<f32>,

    /// Maximum tokens in response
    #[arg(long)]
    pub max_tokens: Option<u32>,

    /// Top-p sampling parameter (0.0-1.0)
    #[arg(long)]
    pub top_p: Option<f32>,

    /// Top-k sampling parameter
    #[arg(long)]
    pub top_k: Option<u32>,

    /// Custom system prompt
    #[arg(short = 's', long)]
    pub system: Option<String>,

    /// Enable streaming mode
    #[arg(long)]
    pub stream: bool,

    /// Interactive REPL mode
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// Text User Interface mode
    #[arg(long)]
    pub tui: bool,

    /// API key (or set ECHOMIND_API_KEY env var)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Request timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Verbose output for debugging
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Initialize default configuration file
    #[arg(long)]
    pub init_config: bool,

    /// Show configuration file path and contents
    #[arg(long)]
    pub show_config: bool,

    /// List all available AI providers
    #[arg(long, alias("ls"))]
    pub list_providers: bool,

    /// Read input from clipboard
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

    /// Optional prompt to append to piped input
    #[arg(value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Use a predefined conversation preset from config
    #[arg(long)]
    pub preset: Option<String>,

    /// Process multiple queries from a file, one per line
    #[arg(long)]
    pub batch: Option<String>,

    /// Enable audit logging
    #[arg(long)]
    pub audit_log: bool,

    /// Generate shell completion script
    #[arg(long, value_enum)]
    pub generate_completion: Option<clap_complete::Shell>,
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
