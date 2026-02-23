# echomind

![Echomind Animation](https://raw.githubusercontent.com/thepinak503/echomind/master/Echomind.gif)


A powerful, lightweight CLI tool in Rust for AI chat APIs. Pipe input, get responses, with streaming, interactive mode, and more.

**Version:** 0.3.5

## ✨ Features
- **Multiple Providers**: OpenAI, Claude, Gemini, Ollama, Grok, Mistral, Cohere, ChatAnywhere, ch.at
- **Streaming & Interactive**: Real-time responses, REPL mode
- **TUI Chat Interface**: WhatsApp-like chat with encrypted persistent history
- **Advanced Options**: Temperature, max tokens, top_p, top_k, model selection
- **Utilities**: Clipboard, file I/O, history, multi-model comparison
- **Cross-Platform**: Linux, macOS, Windows
- **Response Metrics**: Displays provider, model, parameters, and time in ASCII table

## 🖥️ Platform Support

| Platform | Architectures | Status | Installation |
|----------|--------------|--------|--------------|
| **Linux** | x86_64, ARM64 | ✅ Fully Supported | AUR, Cargo, Binary |
| **macOS** | x86_64 (Intel), ARM64 (Apple Silicon) | ✅ Fully Supported | Homebrew, Cargo, Binary |
| **Windows** | x86_64 | ✅ Fully Supported | Scoop, Cargo, Binary |
| **WSL** | x86_64 | ✅ Fully Supported | Same as Linux |

### Pre-built Binaries

Download pre-built binaries for your platform from the [Releases](https://github.com/thepinak503/echomind/releases) page:

- **Linux:** `echomind-linux-amd64.tar.gz`, `echomind-linux-arm64.tar.gz`
- **macOS:** `echomind-macos-amd64.tar.gz`, `echomind-macos-arm64.tar.gz`
- **Windows:** `echomind-windows-amd64.zip`

## 📦 Installation

### Install with Cargo (Recommended)

If you have Rust installed:

```bash
cargo install --git https://github.com/thepinak503/echomind.git --features voice,images
```

### 🚀 Quick Install (Universal)

**Fastest - Linux x86_64 (One-liner):**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/curl-install.sh | bash
```

**Direct download to /usr/bin:**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/echomind-linux-x86_64.gz | gunzip | sudo tee /usr/bin/echomind > /dev/null && sudo chmod +x /usr/bin/echomind
```

**Alternative (full URL):**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

**Windows:**
```powershell
# PowerShell One-liner (Recommended)
irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex
```

### Platform-Specific Installation

#### 🐧 Linux

**Universal (All Distros):**
```bash
cargo install --git https://github.com/thepinak503/echomind.git --features voice,images
```

**Arch Linux (AUR):**
```bash
# Using yay
yay -S echomind

# Using paru
paru -S echomind

# Or manually from AUR
git clone https://aur.archlinux.org/echomind.git
cd echomind
makepkg -si
```

**Debian/Ubuntu (.deb packages):**
```bash
# From releases page (coming soon)
wget https://github.com/thepinak503/echomind/releases/download/v0.3.5/echomind_0.3.5-1_amd64.deb
sudo dpkg -i echomind_*.deb

# Or build from source
cd packaging/debian
dpkg-buildpackage -us -uc -b
```

**Fedora/RHEL (.rpm packages):**
```bash
# From releases page (coming soon)
wget https://github.com/thepinak503/echomind/releases/download/v0.3.5/echomind-0.3.5-1.x86_64.rpm
sudo rpm -i echomind-*.rpm

# Or build from source
cd packaging/fedora
rpmbuild -bb echomind.spec
```

**Fedora/RHEL (.rpm package coming soon):**
```bash
# Download .rpm from releases
rpm -i echomind-*.rpm
```

#### 🍎 macOS

**Supported:** macOS 10.15+ (Catalina, Big Sur, Monterey, Ventura, Sonoma, Sequoia)
**Architecture:** Intel (x86_64) and Apple Silicon (M1/M2/M3/M4 arm64)

**Homebrew (coming soon):**
```bash
brew tap thepinak503/echomind
brew install echomind
```

**From Binary:**
```bash
# Download and extract
curl -LO https://github.com/thepinak503/echomind/releases/download/v0.3.5/echomind-macos-$(uname -m).tar.gz
tar xzf echomind-macos-*.tar.gz
sudo mv echomind /usr/local/bin/
```

**With Cargo:**
```bash
cargo install --git https://github.com/thepinak503/echomind.git --features voice,images
```

#### 🪟 Windows

**Supported:** Windows 10 (version 1809+), Windows 11, Windows Server 2019+
**PowerShell:** 5.1+ or PowerShell Core 7+
**Architecture:** x86_64 (AMD64), ARM64 support coming soon

**PowerShell (One-liner):**
```powershell
irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex
```

**Scoop:**
```powershell
scoop install echomind
# Update:
scoop update echomind
# Uninstall:
scoop uninstall echomind
```

**Chocolatey:**
```powershell
choco install echomind
# Update:
choco upgrade echomind
# Uninstall:
choco uninstall echomind
```

**From Binary:**
```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/thepinak503/echomind/releases/download/v0.3.5/echomind-windows-amd64.zip" -OutFile "echomind.zip"
Expand-Archive -Path "echomind.zip" -DestinationPath "$env:LOCALAPPDATA\Programs\echomind"
# Add to PATH: $env:LOCALAPPDATA\Programs\echomind
```

**With Cargo:**
```bash
cargo install --git https://github.com/thepinak503/echomind.git --features voice,images
```

**WSL (Windows Subsystem for Linux):**
```bash
cargo install --git https://github.com/thepinak503/echomind.git --features voice,images
```

**Windows Terminal Integration:**
Add to Windows Terminal settings for enhanced experience with colors and Unicode support.

### 📦 Pre-built Binaries

Pre-built binaries for all supported platforms are available from the [Releases](https://github.com/thepinak503/echomind/releases) page.

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

## 🔧 Build Dependencies

### Arch Linux / Manjaro
```bash
sudo pacman -S --needed rust cargo openssl pkg-config clang gcc make
```

### Debian / Ubuntu / Linux Mint / Pop!_OS
```bash
sudo apt update
sudo apt install -y rustc cargo libssl-dev pkg-config clang gcc make build-essential
```

### Fedora / RHEL / CentOS / Rocky Linux / AlmaLinux
```bash
sudo dnf install -y rust cargo openssl-devel pkg-config clang gcc make
# Or for RHEL/CentOS with EPEL:
sudo dnf install -y rust-toolset openssl-devel pkg-config clang gcc make
```

### OpenSUSE / SLES
```bash
sudo zypper install -y rust cargo openssl-devel pkg-config clang gcc make
```

### Alpine Linux
```bash
sudo apk add --no-cache rust cargo openssl-dev pkgconfig clang gcc make musl-dev
```

### Gentoo
```bash
sudo emerge --ask dev-lang/rust dev-libs/openssl sys-devel/clang sys-devel/gcc
```

### Void Linux
```bash
sudo xbps-install -S rust cargo openssl-devel pkg-config clang gcc make
```

### NixOS
```nix
# Add to configuration.nix or use nix-shell
nix-shell -p rustc cargo openssl pkg-config clang gcc
```

### Solus
```bash
sudo eopkg install -y rustc cargo openssl-dev pkg-config clang gcc make
```

### 🍎 macOS
```bash
# Using Homebrew
brew install rust openssl pkg-config

# Or install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 🪟 Windows
```powershell
# Install via winget
winget install Rustlang.Rust.MSVC

# Or via Chocolatey
choco install rust

# Or download from https://rustup.rs/
# Also install Visual Studio Build Tools (for MSVC)
```

### Minimal Build (Any Platform)
```bash
# Only essential dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release
```

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

