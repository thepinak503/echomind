use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "echomind")]
#[command(version)]
#[command(about = "Send piped input to AI chat API and print response")]
#[command(
    long_about = "A powerful, cross-platform AI-powered CLI tool with extensive features for integrating AI assistance into your workflow.

Examples:
  echo 'Hello, how are you?' | echomind
  cat file.txt | echomind
  echo 'write a Python function' | echomind --coder --output code.py
  echomind --interactive
  echomind --init-config
  echo 'explain quantum computing' | echomind --provider openai --model gpt-4

Features:
  • Multiple AI providers (OpenAI, Claude, Gemini, Ollama, Grok, Mistral, Cohere, ChatAnywhere, ch.at)
  • Interactive REPL mode with conversation history
  • Streaming responses in real-time
  • Multimodal support (images, PDFs, documents)
  • Voice input/output capabilities
  • Batch processing from files
  • Model comparison and benchmarking
  • Clipboard integration
  • Custom output formatting (JSON, templates)
  • Conversation presets and templates
  • Cross-platform: Linux, macOS, Windows, WSL
  • Performance optimized with async I/O and caching"
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

    // /// Image file to include with the request (for vision models)
    // #[arg(long)]
    // pub image: Option<String>,

    /// Optional prompt to append to input (useful when piping)
    #[arg(value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Use a predefined conversation preset from the config file
    #[arg(long)]
    pub preset: Option<String>,

    /// Process multiple queries from a file, one per line
    #[arg(long)]
    pub batch: Option<String>,

    // Voice features (disabled)
    // /// Enable voice input from microphone
    // #[arg(long)]
    // pub voice_input: bool,

    // /// Convert AI response to speech
    // #[arg(long)]
    // pub voice_output: bool,

    // /// Voice to use for text-to-speech
    // #[arg(long)]
    // pub voice: Option<String>,

    // History management
    /// Search through conversation history
    #[arg(long)]
    pub search_history: Option<String>,

    /// Export history to format (json, csv, markdown)
    #[arg(long)]
    pub export_history: Option<String>,

    /// Show history statistics
    #[arg(long)]
    pub history_stats: bool,

    /// Merge multiple history files
    #[arg(long)]
    pub merge_history: Option<Vec<String>>,

    // Multimodal features (disabled)
    // /// Capture image from webcam
    // #[arg(long)]
    // pub webcam: bool,

    // /// Take screenshot
    // #[arg(long)]
    // pub screenshot: bool,

    // /// Process PDF file
    // #[arg(long)]
    // pub pdf: Option<String>,

    // /// Process Office document
    // #[arg(long)]
    // pub document: Option<String>,

    // /// Process batch images from directory
    // #[arg(long)]
    // pub batch_images: Option<String>,

    // Workflow features
    /// Execute workflow from file
    #[arg(long)]
    pub workflow: Option<String>,

    /// List available workflows
    #[arg(long)]
    pub list_workflows: bool,

    // Collaboration features
    /// Share conversation
    #[arg(long)]
    pub share: bool,

    /// Start collaboration session
    #[arg(long)]
    pub collaborate: bool,

    // Security features
    /// Encrypt conversation
    #[arg(long)]
    pub encrypt: bool,

    /// Local-only mode (no network)
    #[arg(long)]
    pub local_only: bool,

    /// Enable audit logging
    #[arg(long)]
    pub audit_log: bool,

    // Performance features
    /// Benchmark model performance
    #[arg(long)]
    pub benchmark: bool,

    /// Compare model performance
    #[arg(long)]
    pub benchmark_compare: Option<Vec<String>>,

    // Developer tools
    /// Enable debug mode
    #[arg(long)]
    pub debug: bool,

    /// Enable test mode with mock responses
    #[arg(long)]
    pub test_mode: bool,

    // Content management
    /// Use template
    #[arg(long)]
    pub template: Option<String>,

    /// Use snippet
    #[arg(long)]
    pub snippet: Option<String>,

    /// List available snippets
    #[arg(long)]
    pub list_snippets: bool,

    // Data processing
    /// Process CSV file
    #[arg(long)]
    pub csv: Option<String>,

    /// Process JSON file
    #[arg(long)]
    pub json_file: Option<String>,

    /// Process Excel file
    #[arg(long)]
    pub excel: Option<String>,

    // Scheduling
    /// Schedule task for later execution
    #[arg(long)]
    pub schedule: Option<String>,

    // Quality assurance
    /// Enable response quality scoring
    #[arg(long)]
    pub quality_score: bool,

    /// Enable fact-checking
    #[arg(long)]
    pub fact_check: bool,

    /// Enable bias detection
    #[arg(long)]
    pub bias_detect: bool,
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
