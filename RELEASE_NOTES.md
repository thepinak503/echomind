# 🎉 Echomind v0.3.0 - Major Release

**A powerful AI-powered CLI tool with multi-platform support!**

## 🌟 What is Echomind?

Echomind is a lightweight, fast command-line tool written in Rust that pipes input to AI chat APIs and outputs responses. Perfect for integrating AI assistance into your shell workflows!

---

## ✨ New Features

### 🔌 Multiple API Provider Support
- **ChatAnywhere** (`api.chatanywhere.tech`)
- **OpenAI** (GPT-3.5, GPT-4)
- **Claude** (Anthropic)
- **Ollama** (Local LLMs)
- **Custom endpoints** (bring your own API)
- Still supports free **ch.at** API

```bash
echo "Hello!" | echomind --provider openai --model gpt-4
echo "Help me" | echomind --provider ollama --model llama2
```

### 💬 Interactive REPL Mode
Multi-turn conversations without piping!

```bash
echomind --interactive
# Or with streaming:
echomind -i --stream
```

### 🌊 Streaming Responses
Real-time response display as the AI generates text (like ChatGPT typing effect)

```bash
echo "Tell me a story" | echomind --stream
```

### ⚙️ Configuration System
Save your defaults in `~/.config/echomind/config.toml`

```bash
echomind --init-config
```

Example config:
```toml
[api]
provider = "chatanywhere"
api_key = "your-key-here"
model = "gpt-3.5-turbo"
timeout = 30

[defaults]
temperature = 0.7
stream = false
```

### 🎨 Enhanced User Experience
- **Progress indicators** with spinners
- **Colored output** for better readability
- **User-friendly error messages** with actionable advice
- **Help display** when run without input

### 🛠️ Advanced AI Parameters
Fine-tune your AI interactions:

```bash
# Control creativity (temperature)
echo "Be creative!" | echomind --temperature 1.5

# Limit response length
echo "Brief answer" | echomind --max-tokens 100

# Custom system prompt
echo "Hello" | echomind --system "You are a pirate"

# Combine multiple parameters
echo "Code review" | echomind -p openai -m gpt-4 -t 0.3 --stream
```

### 💻 Coder Mode Improvements
Now automatically removes markdown code fences!

```bash
echo "write Python hello world" | echomind --coder --output hello.py
echo "create REST API" | echomind -co api.py
```

---

## 📦 Installation

### 🐧 Linux

**Arch Linux:**
```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
makepkg -si
```

**Debian/Ubuntu:**
```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
sudo apt install -y debhelper cargo rustc libssl-dev pkg-config
dpkg-buildpackage -us -uc -b
sudo dpkg -i ../echomind_0.3.0-1_amd64.deb
```

**Universal Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

### 🍎 macOS

```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

### 🪟 Windows

**PowerShell (Recommended):**
```powershell
irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex
```

**Using Cargo:**
```powershell
cargo install --git https://github.com/thepinak503/echomind
```

**WSL:**
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

---

## 📖 Quick Start

### 1. Install echomind (see above)

### 2. Initialize Configuration
```bash
echomind --init-config
```

### 3. Set Your API Key (if using ChatAnywhere, OpenAI, or Claude)

Edit `~/.config/echomind/config.toml`:
```toml
[api]
provider = "chatanywhere"
api_key = "your-api-key-here"
```

Or use environment variable:
```bash
export ECHOMIND_API_KEY="your-key-here"
```

### 4. Try It Out!

**Simple query:**
```bash
echo "Say hello in 3 languages" | echomind
```

**Interactive mode:**
```bash
echomind --interactive
```

**Generate code:**
```bash
echo "write a bash script to check disk space" | echomind -c -o check_disk.sh
```

**With streaming:**
```bash
echo "Explain quantum computing" | echomind --stream
```

---

## 🆕 New CLI Options

| Option | Short | Description |
|--------|-------|-------------|
| `--provider` | `-p` | Select API provider (chat, chatanywhere, openai, claude, ollama) |
| `--model` | `-m` | Choose model (gpt-3.5-turbo, gpt-4, claude-3-opus, etc.) |
| `--temperature` | `-t` | Control randomness (0.0-2.0) |
| `--max-tokens` | | Limit response length |
| `--system` | `-s` | Custom system prompt |
| `--stream` | | Stream responses in real-time |
| `--interactive` | `-i` | Interactive REPL mode |
| `--api-key` | | API key (or use ECHOMIND_API_KEY env var) |
| `--timeout` | | Request timeout in seconds |
| `--verbose` | `-v` | Enable debug output |
| `--init-config` | | Create default config file |
| `--show-config` | | Display config file location and contents |

---

## 🏗️ Technical Improvements

### Architecture
- **Modular code structure**: Separated into `api.rs`, `cli.rs`, `config.rs`, `error.rs`, `repl.rs`
- **Custom error types**: User-friendly, specific error messages
- **Async throughout**: Better performance with async I/O
- **Proper timeout handling**: With user feedback

### Testing
- **10 unit tests** (all passing)
- **Zero compiler warnings**
- **Integration tests** for API and config
- **CI/CD pipeline** with GitHub Actions

### Dependencies
- Added: `toml`, `dirs`, `thiserror`, `colored`, `rustyline`, `indicatif`, `futures`
- Updated to Rust 2021 edition
- All dependencies up-to-date

### Cross-Platform Support
- **Linux**: Arch (PKGBUILD), Debian/Ubuntu (.deb), universal (install.sh)
- **macOS**: Native support with install.sh
- **Windows**: PowerShell installer (install.ps1)

---

## 📚 Documentation

- **README.md**: Comprehensive usage guide with examples
- **INSTALL.md**: Detailed installation instructions for all platforms
- **CONTRIBUTING.md**: Guidelines for contributors
- **CHANGELOG.md**: Version history
- **Man page**: Updated `echomind.1` with all features
- **Example config**: `config.example.toml` with comments

---

## 🔄 Migration from v0.2.0

### Breaking Changes
None! All existing commands work as before.

### What's Changed
- Configuration file location: `~/.config/echomind/config.toml` (auto-created)
- Install location on Linux: `/usr/bin/` (pacman/apt) instead of `/usr/local/bin/`

### New Defaults
- Temperature: 0.7
- Timeout: 30 seconds
- Provider: "chat" (ch.at - no API key needed)

---

## 💡 Usage Examples

### Basic Usage
```bash
# Simple question
echo "What is Rust?" | echomind

# Process file
cat README.md | echomind "Summarize this"

# Command output
git diff | echomind "Explain these changes"
```

### Code Generation
```bash
# Python function
echo "write factorial function" | echomind -c -o factorial.py

# Web scraper
echo "create web scraper with BeautifulSoup" | echomind --coder

# Shell script
echo "script to backup database" | echomind -co backup.sh
```

### Interactive Sessions
```bash
# Start interactive mode
echomind --interactive

# With streaming for better UX
echomind -i --stream

# With specific provider
echomind -i --provider openai --model gpt-4
```

### Advanced Usage
```bash
# Creative writing (high temperature)
echo "Write a poem" | echomind -t 1.8

# Precise answers (low temperature)
echo "Calculate factorial of 10" | echomind -t 0.1

# Use local LLM
echo "Help me" | echomind --provider ollama --model llama2

# Custom endpoint
echo "Question?" | echomind --provider https://my-api.com/v1/chat
```

---

## 🎯 Supported Providers

| Provider | Endpoint | API Key Required | Models |
|----------|----------|------------------|--------|
| **chat** | ch.at | ❌ No | any |
| **chatanywhere** | api.chatanywhere.tech | ✅ Yes | gpt-3.5-turbo, gpt-4 |
| **openai** | api.openai.com | ✅ Yes | gpt-3.5-turbo, gpt-4, gpt-4-turbo |
| **claude** | api.anthropic.com | ✅ Yes | claude-3-opus, claude-3-sonnet |
| **ollama** | localhost:11434 | ❌ No | llama2, mistral, codellama, etc. |
| **custom** | Your URL | Depends | Any |

---

## 🐛 Bug Fixes

- Fixed input handling when stdin is empty
- Fixed progress indicator conflicts with streaming mode
- Fixed markdown code fence removal in coder mode
- Improved error messages for network issues
- Better handling of API timeouts

---

## 📊 Statistics

- **Lines of code**: 2,832+ added
- **New files**: 20
- **Tests**: 10 (all passing)
- **Compiler warnings**: 0
- **Supported platforms**: 3 (Linux, macOS, Windows)
- **Supported API providers**: 6

---

## 🙏 Acknowledgments

Thanks to:
- **ch.at** and **ChatAnywhere** for providing accessible AI APIs
- The **Rust community** for amazing libraries
- All contributors and testers

---

## 🔗 Links

- **Repository**: https://github.com/thepinak503/echomind
- **Documentation**: See README.md and INSTALL.md
- **Report Issues**: https://github.com/thepinak503/echomind/issues
- **Discussions**: https://github.com/thepinak503/echomind/discussions

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Enjoy using echomind! 🎊**

If you find it useful, please ⭐ star the repository on GitHub!

For questions or feedback, open an issue or start a discussion.
