# echomind

A powerful, lightweight CLI tool in Rust for AI chat APIs. Pipe input, get responses, with streaming, interactive mode, and more.

**Version:** 0.3.2

## ✨ Features
- **Multiple Providers**: OpenAI, Claude, Gemini, Ollama, Grok, Mistral, Cohere, ChatAnywhere, ch.at
- **Streaming & Interactive**: Real-time responses, REPL mode
- **TUI Chat Interface**: WhatsApp-like chat with encrypted persistent history
- **Advanced Options**: Temperature, max tokens, top_p, top_k, model selection
- **Utilities**: Clipboard, file I/O, history, multi-model comparison
- **Cross-Platform**: Linux, macOS, Windows
- **Response Metrics**: Displays provider, model, parameters, and time in ASCII table

## 📦 Installation

### Install with Cargo (Recommended)

If you have Rust installed:

```bash
cargo install --git https://github.com/thepinak503/echomind.git
```

### 🚀 Quick Install (Universal)

**One-liner for Linux/macOS:**
```bash
bash <(curl -fsSL https://is.gd/echomindlin)
```

**Alternative (full URL):**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

**Windows PowerShell:**
```powershell
irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex
```

### Platform-Specific Installation

#### 🐧 Linux

```bash
cargo install --git https://github.com/thepinak503/echomind.git
```


#### 🍎 macOS

**Supported:** macOS 10.15+ (Catalina, Big Sur, Monterey, Ventura, Sonoma, Sequoia)
**Architecture:** Intel (x86_64) and Apple Silicon (M1/M2/M3/M4 arm64)

```bash
cargo install --git https://github.com/thepinak503/echomind.git
```

#### 🪟 Windows

**Supported:** Windows 10 (version 1809+), Windows 11, Windows Server 2019+
**PowerShell:** 5.1+ or PowerShell Core 7+
**Architecture:** x86_64 (AMD64), ARM64 support coming soon

```bash
cargo install --git https://github.com/thepinak503/echomind.git
```

**WSL (Windows Subsystem for Linux):**
```bash
cargo install --git https://github.com/thepinak503/echomind.git
```

**Windows Terminal Integration:**
Add to Windows Terminal settings for enhanced experience with colors and Unicode support.

### 📦 Pre-built Binaries

Pre-built binaries for Linux (x64, musl), macOS (Intel, Apple Silicon), and Windows will be available from the [Releases](https://github.com/thepinak503/echomind/releases) page.

### 🔧 From Source

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Binary will be at: target/release/echomind
```

**📖 For detailed installation instructions for all platforms, see [INSTALL.md](INSTALL.md)**

## 🚀 Usage

### Examples
```bash
# Basic usage
echo "Hello, how are you?" | echomind

# With prompt
ls | echomind "Explain these files"

# Interactive mode
echomind --interactive

# Coder mode
echo "write a Python factorial function" | echomind --coder --output factorial.py

# Different providers
echo "Explain Docker" | echomind --provider openai --model gpt-4

# Streaming
echo "Write a story" | echomind --stream

# Clipboard
echomind --clipboard  # Read from clipboard

# History
echo "What is 2+2?" | echomind --history chat.json

# Compare models
echo "Explain AI" | echomind --compare gpt-4,claude-3-opus

# Advanced parameters
echo "Be creative" | echomind --temperature 1.5 --max-tokens 500 --top-p 0.9
```

See [ENHANCED_FEATURES.md](ENHANCED_FEATURES.md) for all options.

## 📚 Links
- [Installation Guide](INSTALL.md)
- [Contributing](CONTRIBUTING.md)
- [License](LICENSE)

### Initialize Config

Create a default configuration file:

```bash
echomind --init-config
```

This creates `~/.config/echomind/config.toml` with default settings.

### View Config

```bash
echomind --show-config
```

### Configuration File

Edit `~/.config/echomind/config.toml`:

```toml
[api]
provider = "chat"              # Default provider
api_key = "your-key-here"      # API key (if needed)
model = "gpt-3.5-turbo"        # Default model
timeout = 30                   # Request timeout in seconds

[defaults]
temperature = 0.7              # Default temperature
max_tokens = 2000              # Default max tokens
coder_mode = false             # Enable coder mode by default
stream = false                 # Enable streaming by default
```

### Environment Variables

Set API key via environment variable:

```bash
export ECHOMIND_API_KEY="your-api-key"
echo "Hello" | echomind --provider openai
```

## 📖 Options

| Option | Short | Description |
|--------|-------|-------------|
| `--coder` | `-c` | Enable coder mode (clean code output) |
| `--output <FILE>` | `-o` | Save response to file |
| `--co <FILE>` | | Combined `--coder --output` |
| `--provider <NAME>` | `-p` | API provider (chat, chatanywhere, openai, claude, ollama) |
| `--model <MODEL>` | `-m` | Model to use |
| `--temperature <NUM>` | `-t` | Temperature (0.0-2.0) |
| `--max-tokens <NUM>` | | Maximum tokens in response |
| `--system <PROMPT>` | `-s` | Custom system prompt |
| `--stream` | | Stream response as it arrives |
| `--interactive` | `-i` | Interactive REPL mode |
| `--clipboard` | | Read input from clipboard |
| `--to-clipboard` | | Save response to clipboard |
| `--history <FILE>` | | Conversation history file for persistent context |
| `--compare <MODELS>` | | Compare responses from multiple models (comma-separated) |
| `--format <FORMAT>` | | Output format: text, json, or template:<template> |
| `--api-key <KEY>` | | API key for provider |
| `--timeout <SECS>` | | Request timeout in seconds |
| `--verbose` | `-v` | Enable verbose output |
| `--init-config` | | Initialize default config file |
| `--show-config` | | Show config file location and contents |
| `<PROMPT>` | | Optional prompt to append to piped input |
| `--help` | `-h` | Display help information |
| `--version` | `-V` | Display version information |

## 🔌 Supported Providers

| Provider | Endpoint | API Key Required |
|----------|----------|------------------|
| chat | `https://ch.at/v1/chat/completions` | No |
| chatanywhere | `https://api.chatanywhere.tech/v1/chat/completions` | Yes |
| openai | `https://api.openai.com/v1/chat/completions` | Yes |
| claude | `https://api.anthropic.com/v1/messages` | Yes |
| ollama | `http://localhost:11434/api/chat` | No |
| grok | `https://api.x.ai/v1/chat/completions` | Yes |
| mistral | `https://api.mistral.ai/v1/chat/completions` | Yes |
| cohere | `https://api.cohere.ai/v1/chat` | Yes |
| custom | Your custom URL | Depends |

## 💡 Examples

### Shell Integration

```bash
# Explain command output
ls -la | echomind "Explain these files"
df -h | echomind "Analyze disk usage"

# Debug error messages
./script.sh 2>&1 | echomind "What's wrong with this error?"

# Git workflows
git status | echomind "Summarize changes"
git log --oneline -10 | echomind "Explain recent commits"
git diff | echomind "Review this code"

# System information
ps aux | echomind "Show top processes"
netstat -tulpn | echomind "Explain open ports"

# Generate code from description
echo "Create a Rust function to parse JSON" | echomind -co parser.rs

# Review code
cat main.rs | echomind "Review this code for security issues"

# Translate text
echo "Hello world" | echomind -s "Translate to Spanish"
```

### CI/CD Integration

```bash
# Automated code review in CI
git diff main | echomind "Review changes for security issues" > review.txt
```

### Quick Scripts

```bash
# One-liner to generate and run Python code
echo "script to download a file from URL" | echomind -co download.py && python download.py
```

## 🛠️ Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy
cargo fmt
```

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 Dependencies

- `reqwest` - HTTP client for API requests
- `serde` - JSON serialization/deserialization
- `tokio` - Async runtime
- `clap` - Command-line argument parsing
- `toml` - Configuration file parsing
- `dirs` - Cross-platform config directory
- `thiserror` - Error handling
- `indicatif` - Progress indicators
- `colored` - Terminal colors
- `rustyline` - Interactive REPL
- `futures` - Async streaming

## 🙏 Credits

Thanks to the ch.at API and ChatAnywhere for providing accessible AI endpoints. This project leverages the power of AI chat completions to make command-line interactions more intelligent and productive. Special shoutout to the open-source community for providing the libraries that make Rust development a breeze.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🐛 Troubleshooting

### API Key Issues

If you see "API key required" error:
```bash
# Set via environment variable
export ECHOMIND_API_KEY="your-key"

# Or pass directly
echomind --api-key "your-key" --provider openai

# Or add to config file
echomind --init-config
# Then edit ~/.config/echomind/config.toml
```

### Timeout Errors

If requests timeout:
```bash
# Increase timeout
echo "question" | echomind --timeout 60

# Or set in config
[api]
timeout = 60
```

### Network Issues

For connection errors, check:
- Internet connectivity
- Firewall settings
- API endpoint status
- Use `--verbose` for debugging


