# Echomind v0.3.0 - Major Release 🎉

A powerful, lightweight AI-powered CLI tool with **multi-platform support** and **multiple API providers**!

## 🌟 Highlights

- ✨ **Multiple API Providers**: ChatAnywhere, OpenAI, Gemini, Claude, Ollama, custom endpoints
- 💬 **Interactive REPL Mode**: Multi-turn conversations with `-i/--interactive`
- 🌊 **Streaming Responses**: Real-time display with `--stream`
- ⚙️ **Configuration System**: Save defaults in `~/.config/echomind/config.toml`
- 🎨 **Enhanced UX**: Progress indicators, colored output, better errors
- 🛠️ **Advanced Parameters**: Temperature, max-tokens, model selection, custom prompts
- 📦 **Cross-Platform**: Linux (Arch, Debian/Ubuntu), macOS, Windows
- 🧪 **Tested**: 10 unit tests, CI/CD, zero warnings

## 📦 Quick Install

### Linux/macOS
```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex
```

### Arch Linux
```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
makepkg -si
```

### Debian/Ubuntu
```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
dpkg-buildpackage -us -uc -b
sudo dpkg -i ../echomind_0.3.0-1_amd64.deb
```

## 🚀 Quick Start

```bash
# Initialize config
echomind --init-config

# Edit config to add your API key
nano ~/.config/echomind/config.toml

# Try it out!
echo "Say hello in 3 languages" | echomind

# Interactive mode
echomind --interactive

# Generate code
echo "write Python factorial" | echomind -c -o factorial.py

# With streaming
echo "Tell me a story" | echomind --stream
```

## 🆕 What's New

### Features
- Multiple API provider support (chat, chatanywhere, openai, gemini, claude, ollama)
- Interactive REPL mode for conversations
- Streaming responses in real-time
- Configuration file support (`~/.config/echomind/config.toml`)
- Advanced AI parameters (temperature, max-tokens, system prompts)
- Progress indicators and colored output
- Better error messages

### CLI Options
- `--provider/-p`: Select API provider
- `--model/-m`: Choose model
- `--temperature/-t`: Control randomness (0.0-2.0)
- `--max-tokens`: Limit response length
- `--system/-s`: Custom system prompt
- `--stream`: Stream responses
- `--interactive/-i`: Interactive mode
- `--api-key`: API key
- `--init-config`: Create config file
- `--verbose/-v`: Debug output

### Installation
- **Debian/Ubuntu**: Full `.deb` package support
- **Windows**: PowerShell installer (`install.ps1`)
- **macOS**: Enhanced `install.sh`
- **Arch**: Improved PKGBUILD

### Technical
- Modular architecture (api, cli, config, error, repl modules)
- Custom error types with helpful messages
- Comprehensive test suite (10 tests)
- CI/CD with GitHub Actions
- Zero compiler warnings

## 📚 Documentation

- **[INSTALL.md](INSTALL.md)**: Detailed installation for all platforms
- **[README.md](README.md)**: Complete usage guide
- **[CONTRIBUTING.md](CONTRIBUTING.md)**: Contribution guidelines
- **[CHANGELOG.md](CHANGELOG.md)**: Version history

## 💡 Example Usage

```bash
# Basic
echo "What is Rust?" | echomind

# Code generation
echo "write bash backup script" | echomind -co backup.sh

# Interactive with streaming
echomind -i --stream

# Use OpenAI GPT-4
echo "Explain quantum computing" | echomind -p openai -m gpt-4

# Local Ollama
echo "Help me" | echomind --provider ollama --model llama2

# Creative writing
echo "Write a poem" | echomind -t 1.5
```

## 🎯 Supported Providers

| Provider | API Key | Models |
|----------|---------|--------|
| chat (ch.at) | ❌ | gpt-3.5-turbo |
| chatanywhere | ✅ | gpt-3.5-turbo, gpt-4 |
| openai | ✅ | gpt-3.5-turbo, gpt-4 |
| gemini | ✅ | gemini-1.5-pro, gemini-pro |
| claude | ✅ | claude-3-opus, claude-3-sonnet |
| ollama | ❌ | llama2, mistral, codellama |
| custom | Depends | Any |

## 🔄 Migration from v0.2.0

No breaking changes! All existing commands work as before.

New config location: `~/.config/echomind/config.toml` (auto-created with `--init-config`)

## 📊 Stats

- **2,832+ lines added**
- **20 new files**
- **10 tests (all passing)**
- **0 warnings**
- **3 platforms supported**
- **6 API providers**

## 🙏 Credits

Thanks to ch.at, ChatAnywhere, and the Rust community!

---

**Full release notes**: [RELEASE_NOTES.md](RELEASE_NOTES.md)

**Repository**: https://github.com/thepinak503/echomind

**License**: MIT

---

If you find echomind useful, please ⭐ **star the repository**!

Report issues: https://github.com/thepinak503/echomind/issues
