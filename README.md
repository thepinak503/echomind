# echomind

A powerful, lightweight command-line tool written in Rust that pipes input to AI chat APIs and outputs responses. Perfect for integrating AI assistance into your shell workflows with support for multiple providers, streaming responses, and interactive mode.

## ✨ Features

### Core Features
- **Simple piping**: Read from stdin and send to AI
- **Multiple API providers**: Support for 9+ AI providers (OpenAI, Claude, Gemini, Ollama, Grok, Mistral, Cohere, ChatAnywhere, ch.at)
- **Streaming responses**: Real-time response display with `--stream`
- **Interactive REPL mode**: Multi-turn conversations with `-i/--interactive`
- **Coder mode**: Generate clean code with `--coder`
- **File output**: Save responses directly to files with `--output`
- **Clipboard support**: Read from/write to clipboard with `--clipboard` and `--to-clipboard`
- **Conversation history**: Persistent context with `--history <file>`
- **Multi-model comparison**: Compare responses from multiple models with `--compare`
- **Configuration system**: Save defaults in `~/.config/echomind/config.toml`
- **Advanced parameters**: Control temperature, max tokens, model selection
- **Progress indicators**: Visual feedback during API calls
- **Fast and async**: Optimized for performance with async I/O and caching
- **Cross-platform**: Linux, macOS, Windows, WSL (see compatibility below)
- **User-friendly errors**: Clear, actionable error messages

### Advanced Features
- **Multimodal support**: Images, PDFs, documents with `--image`, `--pdf`, `--document`
- **Voice features**: Voice input/output with `--voice-input`, `--voice-output`
- **Batch processing**: Process multiple queries with `--batch`
- **Model benchmarking**: Performance testing with `--benchmark`
- **Output formatting**: JSON, templates with `--format`
- **Conversation presets**: Reusable templates with `--preset`
- **Workflow automation**: Execute complex workflows with `--workflow`
- **Collaboration**: Share conversations with `--share`, `--collaborate`
- **Data processing**: CSV, JSON, Excel with `--csv`, `--json-file`, `--excel`
- **Quality assurance**: Fact-checking, bias detection with `--fact-check`, `--bias-detect`
- **Security features**: Encryption, audit logging with `--encrypt`, `--audit-log`
- **Scheduling**: Time-based task execution with `--schedule`

## 💻 Cross-Platform Compatibility

| Operating System | Versions | Status | Architecture | Package Manager | Notes |
|-----------------|----------|--------|--------------|-----------------|-------|
| **Linux** | | | | | |
| ├─ **Ubuntu** | 20.04+ | ✅ Fully Supported | x86_64, ARM64 | apt, snap | .deb packages, universal installer |
| ├─ **Debian** | 11+ | ✅ Fully Supported | x86_64, ARM64 | apt | .deb packages available |
| ├─ **Arch Linux** | Rolling | ✅ Fully Supported | x86_64 | pacman | PKGBUILD in AUR |
| ├─ **Fedora** | 35+ | ✅ Fully Supported | x86_64, ARM64 | dnf | RPM packages |
| ├─ **CentOS/RHEL** | 8+ | ✅ Fully Supported | x86_64 | yum/dnf | RPM packages |
| ├─ **openSUSE** | Leap 15.3+, Tumbleweed | ✅ Fully Supported | x86_64 | zypper | RPM packages |
| ├─ **Alpine Linux** | 3.14+ | ✅ Supported | x86_64, ARM64 | apk | Manual installation |
| └─ **Other Linux** | Any with Rust 1.70+ | ✅ Supported | x86_64, ARM64 | - | Universal installer |
| **macOS** | | | | | |
| ├─ **macOS** | 10.15+ (Catalina+) | ✅ Fully Supported | Intel x86_64 | Homebrew | Universal binaries |
| └─ **macOS** | 11.0+ (Big Sur+) | ✅ Fully Supported | Apple Silicon ARM64 | Homebrew | Native ARM64 support |
| **Windows** | | | | | |
| ├─ **Windows 10** | 1809+ | ✅ Fully Supported | x86_64 | WinGet, Chocolatey | PowerShell installer |
| ├─ **Windows 11** | 21H2+ | ✅ Fully Supported | x86_64, ARM64 | WinGet, Chocolatey | PowerShell installer |
| └─ **Windows Server** | 2019+ | ✅ Supported | x86_64 | - | Manual installation |
| **Containers** | | | | | |
| ├─ **Docker** | All versions | ✅ Supported | Multi-arch | Docker Hub | Pre-built images |
| ├─ **Podman** | All versions | ✅ Supported | Multi-arch | - | Compatible with Docker images |
| └─ **WSL** | WSL 1 & WSL 2 | ✅ Fully Supported | x86_64 | apt/pacman/etc | Use Linux installation methods |

**System Requirements:**
- **RAM**: 50 MB minimum, 128 MB recommended
- **Disk Space**: 10 MB (binary only), 500 MB (with Rust toolchain for building)
- **Network**: Internet connection required for API calls (except local Ollama)
- **CPU**: Any modern CPU (x86_64, ARM64, Apple Silicon)
- **OS Kernel**: Linux 3.10+, macOS 10.15+, Windows 10 1809+

**Feature Compatibility:**
- ✅ All core features work on all platforms
- ✅ Voice features require system audio libraries (ALSA/PulseAudio on Linux, CoreAudio on macOS, WASAPI on Windows)
- ✅ Clipboard features use native system APIs
- ✅ File I/O works with platform-specific path handling
- ✅ Terminal colors and formatting work on all modern terminals

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

### Basic Usage

Pipe input to echomind from stdin:

```bash
echo "Hello, how are you?" | echomind
```

Use with other commands and add a prompt:

```bash
# Pipe output and add a prompt
ls | echomind "Explain these files"
cat file.txt | echomind "Summarize this"
git diff | echomind "Review these changes"

# Without prompt
cat file.txt | echomind
```

### Interactive Mode

Start a multi-turn conversation:

```bash
echomind --interactive
echomind -i --stream  # With streaming
```

In interactive mode:
- Type your messages and press Enter
- Use `exit` or `Ctrl+D` to quit
- Use `clear` to reset conversation history

### Coder Mode

Generate code and save to file:

```bash
echo "write a Python function to calculate factorial" | echomind --coder --output factorial.py
echo "create a REST API with FastAPI" | echomind -c -o api.py
```

Combined short options:

```bash
echo "optimize this SQL query" | echomind -co optimized.sql
```

### Multiple API Providers

Use different AI providers with their respective models:

```bash
# Free providers (no API key needed)
echo "Hello" | echomind --provider chat                    # ch.at API
echo "Help" | echomind --provider ollama --model llama2    # Local Ollama

# Commercial providers (require API keys)
echo "Explain Docker" | echomind --provider openai --model gpt-4
echo "Write a poem" | echomind --provider claude --model claude-3-opus
echo "Analyze this" | echomind --provider gemini --model gemini-1.5-pro
echo "Explain memes" | echomind --provider grok --model grok-1
echo "Write a story" | echomind --provider mistral --model mistral-large
echo "Summarize text" | echomind --provider cohere --model command

# Custom endpoints
echo "Question?" | echomind --provider https://your-api.com/v1/chat/completions
```

### Streaming Responses

Display responses as they arrive:

```bash
echo "Write a long essay about AI" | echomind --stream
```

### Clipboard Support

Read from and write to clipboard:

```bash
# Read from clipboard (instead of stdin)
echomind --clipboard

# Or on macOS
pbpaste | echomind

# Save response to clipboard
echo "Explain Docker" | echomind --to-clipboard

# Both: read from and write to clipboard
echomind --clipboard --to-clipboard
```

### Conversation History

Maintain context across multiple queries:

```bash
# First query with history
echo "What is 2+2?" | echomind --history chat.json

# Follow-up query with same history
echo "What about multiplied by 3?" | echomind --history chat.json

# Review history
cat chat.json
```

### Multi-Model Comparison

Compare responses from multiple models:

```bash
# Compare GPT-4 and Claude
echo "Explain quantum computing" | echomind --compare gpt-4,claude-3-opus

# Compare local and cloud models
echo "Write a poem" | echomind --compare ollama/llama2,gpt-3.5-turbo

# Use with clipboard
echomind --clipboard --compare gpt-4,gpt-3.5-turbo,claude-3-sonnet
```

### Advanced Parameters

Fine-tune AI behavior and output:

```bash
# Temperature control (0.0-2.0)
echo "Be creative!" | echomind --temperature 1.5      # High creativity
echo "Be precise" | echomind --temperature 0.1       # Low creativity

# Response length limits
echo "Brief answer" | echomind --max-tokens 50        # Short responses
echo "Detailed analysis" | echomind --max-tokens 2000 # Long responses

# Custom system prompts
echo "Hello" | echomind --system "You are a pirate. Respond in pirate speak."
echo "Code" | echomind --system "You are an expert programmer. Explain concepts clearly."

# Output formatting
echo "List items" | echomind --format json
echo "Question" | echomind --format "template:Q: {prompt}\nA: {content}"

# Combine multiple parameters
echo "Code review" | echomind -p openai -m gpt-4 -t 0.3 --max-tokens 2000 --stream
```

### Multimodal Features

Work with images, documents, and other media:

```bash
# Analyze images
echomind --image diagram.png "Explain this flowchart"
echomind --image photo.jpg "What do you see in this picture?"

# Process documents
echomind --pdf research.pdf "Summarize this paper"
echomind --document report.docx "Extract key points"

# Batch processing
echomind --batch-images ./photos/ "Describe these images"

# Webcam and screenshots
echomind --webcam "What's in front of the camera?"
echomind --screenshot "Analyze this screen"
```

### Voice Features

Voice input and output capabilities:

```bash
# Voice input from microphone
echomind --voice-input "Speak your question"

# Text-to-speech output
echo "Hello world" | echomind --voice-output

# Combined voice interaction
echomind --voice-input --voice-output --voice "alloy"

# Specify voice
echo "Read this text" | echomind --voice-output --voice "nova"
```

### Batch Processing & Automation

Process multiple queries efficiently:

```bash
# Process multiple queries from file
echo -e "What is AI?\nExplain Rust\nWrite hello world in Python" > queries.txt
echomind --batch queries.txt

# Workflow automation
echomind --workflow code-review-workflow.json
echomind --list-workflows

# Scheduling
echomind --schedule "2024-12-25 10:00" "Send holiday greeting"
```

### Model Comparison & Benchmarking

Compare and benchmark different models:

```bash
# Compare multiple models
echo "Pros and cons of microservices" | echomind --compare "gpt-4,claude-3-opus,gemini-pro"

# Benchmark performance
echo "Explain quantum computing" | echomind --benchmark --provider openai --model gpt-4

# Performance comparison
echomind --benchmark-compare "gpt-3.5-turbo,gpt-4,claude-3-haiku" "Complex algorithm explanation"
```

### Data Processing

Work with various data formats:

```bash
# Process CSV files
echomind --csv sales.csv "Analyze this sales data"

# Process JSON data
echomind --json-file config.json "Explain this configuration"

# Process Excel spreadsheets
echomind --excel report.xlsx "Summarize this financial report"
```

### Collaboration & Sharing

Share and collaborate on conversations:

```bash
# Share conversation
echomind --share --history session.json

# Start collaboration session
echomind --collaborate --history team-session.json

# Export conversation history
echomind --export-history markdown --history session.json > conversation.md
```

### Quality Assurance & Security

Advanced quality and security features:

```bash
# Fact checking
echo "The Earth is flat" | echomind --fact-check

# Bias detection
echo "Analyze this text for bias" | echomind --bias-detect

# Quality scoring
echo "Explain photosynthesis" | echomind --quality-score

# Encrypted conversations
echomind --encrypt --history secure-session.json

# Audit logging
echomind --audit-log --history audited-session.json
```

## ⚙️ Configuration

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


