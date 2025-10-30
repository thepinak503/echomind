# Echomind v0.3.0 - Major Release 🎉

A powerful, lightweight AI-powered CLI tool with **multi-platform support** and **multiple API providers**!

## 🌟 Highlights

- ✨ **Multiple API Providers**: ChatAnywhere, OpenAI, Gemini, Claude, Ollama, Grok, Mistral, Cohere, custom endpoints
- 💬 **Interactive REPL Mode**: Multi-turn conversations with `-i/--interactive`
- 🌊 **Streaming Responses**: Real-time display with `--stream`
- 🖼️ **Multimodal Support**: Image input for vision-capable models
- 📝 **Batch Processing**: Process multiple queries from files
- 📋 **Clipboard Integration**: Read from/write to clipboard
- 📚 **Conversation History**: Persistent context across sessions
- ⚖️ **Model Comparison**: Compare responses from multiple models
- 🎨 **Output Formatting**: Custom formatting (text, json, template)
- ⚙️ **Configuration System**: Save defaults in `~/.config/echomind/config.toml`
- 🎨 **Enhanced UX**: Progress indicators, colored output, better errors
- 🛠️ **Advanced Parameters**: Temperature, max-tokens, model selection, custom prompts
- 📦 **Cross-Platform**: Linux (Arch, Debian/Ubuntu), macOS, Windows
- 🧪 **Tested**: 11 unit tests, CI/CD, zero warnings

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
- Multiple API provider support (chat, chatanywhere, openai, gemini, claude, ollama, grok, mistral, cohere)
- Interactive REPL mode for conversations
- Streaming responses in real-time
- Multimodal support with image input for vision models
- Batch processing of multiple queries from files
- Clipboard integration (read from/write to clipboard)
- Conversation history with persistent context
- Model comparison across multiple providers
- Custom output formatting (text, json, templates)
- Conversation presets for common use cases
- Configuration file support (`~/.config/echomind/config.toml`)
- Advanced AI parameters (temperature, max-tokens, system prompts)
- Progress indicators and colored output
- Enhanced error messages with actionable suggestions

### CLI Options
- `--provider/-p`: Select API provider
- `--model/-m`: Choose model
- `--temperature/-t`: Control randomness (0.0-2.0)
- `--max-tokens`: Limit response length
- `--system/-s`: Custom system prompt
- `--stream`: Stream responses
- `--interactive/-i`: Interactive mode
- `--api-key`: API key
- `--clipboard`: Read from clipboard
- `--to-clipboard`: Save to clipboard
- `--history`: Conversation history file
- `--compare`: Compare multiple models
- `--format`: Output format (text, json, template)
- `--image`: Include image for vision models
- `--preset`: Use conversation preset
- `--batch`: Process queries from file
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

# Multimodal (with image)
echomind --image photo.jpg "What's in this image?"

# Batch processing
echo -e "What is AI?\nExplain Rust\nWrite hello world" > queries.txt
echomind --batch queries.txt

# Compare models
echo "Explain recursion" | echomind --compare "gpt-3.5-turbo,claude-3-haiku"

# Clipboard integration
echomind --clipboard --to-clipboard "Summarize this text"

# Custom formatting
echo "List 3 fruits" | echomind --format json

# Conversation history
echomind -i --history mychat.json
```

## 🎯 Supported Providers

| Provider | API Key | Models | Multimodal |
|----------|---------|--------|------------|
| chat (ch.at) | ❌ | gpt-3.5-turbo | ❌ |
| chatanywhere | ✅ | gpt-3.5-turbo, gpt-4 | ❌ |
| openai | ✅ | gpt-3.5-turbo, gpt-4, gpt-4-vision | ✅ |
| gemini | ✅ | gemini-1.5-pro, gemini-pro | ✅ |
| claude | ✅ | claude-3-opus, claude-3-sonnet | ✅ |
| ollama | ❌ | llama2, mistral, codellama, llava | ✅ |
| grok | ✅ | grok-1 | ❌ |
| mistral | ✅ | mistral-large, mistral-medium | ❌ |
| cohere | ✅ | command, command-light | ❌ |
| custom | Depends | Any | Depends |

## 🔄 Migration from v0.2.0

No breaking changes! All existing commands work as before.

New config location: `~/.config/echomind/config.toml` (auto-created with `--init-config`)

## 📊 Stats

- **3,200+ lines of code**
- **25+ new files**
- **11 tests (all passing)**
- **0 warnings**
- **3 platforms supported**
- **9 API providers**
- **Multimodal support**
- **Batch processing**
- **Advanced formatting**

## 🙏 Credits

Thanks to ch.at, ChatAnywhere, and the Rust community!

---

**Full release notes**: [RELEASE_NOTES.md](RELEASE_NOTES.md)

**Repository**: https://github.com/thepinak503/echomind

**License**: MIT

---

If you find echomind useful, please ⭐ **star the repository**!

Report issues: https://github.com/thepinak503/echomind/issues
