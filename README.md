# echomind

A powerful, lightweight command-line tool written in Rust that pipes input to AI chat APIs and outputs responses. Perfect for integrating AI assistance into your shell workflows with support for multiple providers, streaming responses, and interactive mode.

## ✨ Features

- **Simple piping**: Read from stdin and send to AI
- **Multiple API providers**: Support for ch.at, ChatAnywhere, OpenAI, Claude, Ollama, and custom endpoints
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
- **Fast and async**: Optimized for performance with async I/O
- **Cross-platform**: Linux, macOS, Windows (see compatibility below)
- **User-friendly errors**: Clear, actionable error messages

## 💻 OS Compatibility

| Operating System | Versions | Status | Notes |
|-----------------|----------|--------|-------|
| **Ubuntu/Debian** | Ubuntu 20.04+, Debian 11+ | ✅ Fully Supported | .deb packages available |
| **Arch Linux** | Rolling | ✅ Fully Supported | PKGBUILD available |
| **Fedora** | Fedora 35+, RHEL 9+ | ✅ Fully Supported | dnf package manager |
| **CentOS/RHEL** | CentOS 8+, RHEL 8+ | ✅ Fully Supported | yum/dnf supported |
| **openSUSE** | Leap 15.3+, Tumbleweed | ✅ Fully Supported | zypper package manager |
| **Other Linux** | Any with Rust support | ✅ Supported | Manual installation |
| **macOS** | macOS 10.15+ (Catalina+) | ✅ Fully Supported | Intel & Apple Silicon |
| **Windows** | Windows 10 (1809+), 11 | ✅ Fully Supported | PowerShell 5.1+/Core 7+ |
| **WSL** | WSL 1 & WSL 2 | ✅ Fully Supported | Use Linux method |

**Minimum Requirements:**
- **RAM**: 50 MB
- **Disk Space**: 10 MB (binary only), 500 MB (with Rust toolchain)
- **Network**: Internet connection for API calls

## 📦 Installation

### Quick Install (Linux/macOS)

```bash
bash <(curl -fsSL https://is.gd/echomindlin)
```

**Alternative (full URL):**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

### Platform-Specific Installation

#### 🐧 Linux

**Supported Distributions:**
- Ubuntu 20.04+, Debian 11+ (apt)
- Arch Linux, Manjaro (pacman)
- Fedora 35+, RHEL 9+ (dnf)
- CentOS 8+, Rocky Linux, AlmaLinux (yum/dnf)
- openSUSE Leap 15.3+, Tumbleweed (zypper)
**(AUR)**
```bash
yay -S echomind
```
OR
```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
makepkg -si
```
or
**Using Cargo (if Rust is installed):**
```bash
cargo install --git https://github.com/thepinak503/echomind
```


#### 🍎 macOS

**Supported:** macOS 10.15+ (Catalina, Big Sur, Monterey, Ventura, Sonoma)
**Architecture:** Intel (x86_64) and Apple Silicon (M1/M2/M3 arm64)

```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release
sudo install -m 755 target/release/echomind /usr/local/bin/echomind
```

#### 🪟 Windows

**Supported:** Windows 10 (version 1809+), Windows 11
**PowerShell:** 5.1+ or PowerShell Core 7+

**Quick Install (Recommended - Run as Administrator):**
```powershell
# Download the executable
curl -L https://github.com/thepinak503/echomind/raw/master/echomind-windows-x86_64.exe -o echomind.exe

# Move to a directory in PATH (e.g., C:\Windows\System32)
Move-Item echomind.exe C:\Windows\System32\echomind.exe
```

**WSL (Windows Subsystem for Linux):**
WSL 1 & WSL 2 supported - Use the Linux installation instructions

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

Use different AI providers:

```bash
# ChatAnywhere (requires API key)
echo "Hello" | echomind --provider chatanywhere --api-key YOUR_KEY

# OpenAI
echo "Explain Docker" | echomind --provider openai --model gpt-4

# Claude
echo "Write a poem" | echomind --provider claude --model claude-3-opus

# Ollama (local)
echo "Help me" | echomind --provider ollama --model llama2

# Custom endpoint
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

Control AI behavior:

```bash
# Temperature (creativity)
echo "Tell me a story" | echomind --temperature 1.5

# Max tokens (response length)
echo "Explain briefly" | echomind --max-tokens 100

# Custom system prompt
echo "Hello" | echomind --system "You are a pirate. Respond in pirate speak."

# Combine parameters
echo "Code review" | echomind -p openai -m gpt-4 -t 0.3 --max-tokens 2000
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

# Contributing to Echomind

Thank you for your interest in contributing to Echomind! We welcome contributions from the community and are excited to see what you'll help build. This guide will walk you through the process of forking the repository and submitting a pull request.

## Quick Start

1. **Fork** the repository on GitHub
2. **Clone** your fork locally
3. **Create** a feature branch
4. **Make** your changes
5. **Test** your changes
6. **Commit** and **push** to your fork
7. **Open** a Pull Request

## Detailed Instructions

### 1. Fork the Repository

- Visit https://github.com/thepinak503/echomind
- Click the **"Fork"** button in the top-right corner
- This creates your own copy of the repository under your GitHub account

### 2. Clone Your Fork

```bash
# Clone your fork to your local machine
git clone https://github.com/YOUR_USERNAME/echomind.git
cd echomind

# Add the original repository as upstream
git remote add upstream https://github.com/thepinak503/echomind.git
```

### 3. Set Up Development Environment

Make sure you have Rust installed (see [INSTALL.md](INSTALL.md) for detailed setup):

```bash
# Verify Rust installation
rustc --version
cargo --version

# Build the project
cargo build

# Run tests to ensure everything works
cargo test
```

### 4. Create a Feature Branch

```bash
# Sync with upstream main branch
git fetch upstream
git checkout main
git merge upstream/main

# Create a new branch for your feature
git checkout -b feature/your-feature-name
```

**Branch naming conventions:**
- `feature/description` for new features
- `fix/description` for bug fixes
- `docs/description` for documentation
- `refactor/description` for code refactoring

### 5. Make Your Changes

- Write clean, well-documented code
- Follow Rust conventions and style
- Add tests for new functionality
- Update documentation as needed

**Testing your changes:**
```bash
# Run all tests
cargo test

# Run specific integration tests
cargo test --test integration

# Test the CLI directly
cargo run -- --help
```

### 6. Commit Your Changes

```bash
# Stage your changes
git add .

# Commit with a descriptive message
git commit -m "feat: add new provider support for XYZ

- Implement XYZ API provider
- Add corresponding model configurations
- Update documentation with examples
- Add unit tests for new provider

Fixes #123"  # Reference issue numbers if applicable
```

**Commit message guidelines:**
- Use the conventional commits format: `type(scope): description`
- Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
- Keep the first line under 50 characters
- Provide detailed description in the body

### 7. Push to Your Fork

```bash
git push origin feature/your-feature-name
```

### 8. Create a Pull Request

1. Visit your fork on GitHub: `https://github.com/YOUR_USERNAME/echomind`
2. Click **"Compare & pull request"** next to your branch
3. Fill out the PR template:
   - **Title**: Clear, descriptive title
   - **Description**: Detailed explanation of changes
   - **Related Issues**: Link to any relevant issues
   - **Checklist**: Verify all items are completed

## PR Template

When creating your pull request, please use this format:

```markdown
## Description
<!-- Clearly describe what this PR does and why it's needed -->

## Related Issue
<!-- Link to the issue this PR addresses -->
Fixes #issue_number

## Type of Change
- [ ] 🐛 Bug fix
- [ ] ✨ New feature
- [ ] 📚 Documentation update
- [ ] 🎨 UI/UX improvement
- [ ] ⚡ Performance improvement
- [ ] 🔧 Refactor
- [ ] ✅ Test addition

## Testing
<!-- Describe how you tested your changes -->
- [ ] All existing tests pass
- [ ] New tests added
- [ ] Tested on Linux
- [ ] Tested on macOS
- [ ] Tested on Windows

## Checklist
- [ ] Code follows project style guidelines
- [ ] Documentation updated (README, man pages, etc.)
- [ ] Tests added/updated
- [ ] All tests pass locally
- [ ] No new warnings introduced
- [ ] PR title follows conventional commits
- [ ] Commit messages are clear and descriptive

## Screenshots/Screen Recordings
<!-- If applicable, add screenshots or screen recordings -->

## Additional Notes
<!-- Any additional context or notes for reviewers -->
```

## Development Guidelines

### Code Style
- Follow Rust formatting with `cargo fmt`
- Use `cargo clippy` to catch common mistakes
- Document public functions and types
- Use meaningful variable names

### Testing
- Write unit tests for new functionality
- Ensure all tests pass before submitting
- Test on multiple platforms if possible
- Verify CLI functionality manually

### Documentation
- Update README.md for new features
- Update man pages if CLI changes
- Add examples for new functionality
- Document breaking changes clearly

## Need Help?

- Check existing [Issues](https://github.com/thepinak503/echomind/issues)
- Join [Discussions](https://github.com/thepinak503/echomind/discussions)
- Review [CONTRIBUTING.md](CONTRIBUTING.md) for more details

## Recognition

All contributors will be acknowledged in our release notes and contributors list. Thank you for helping make Echomind better! 🎉

---

**Happy coding!** If you have any questions during the process, don't hesitate to ask in the discussions or issues.

---

**Made with ❤️ using Rust**
