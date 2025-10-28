# echomind

A powerful, lightweight command-line tool written in Rust that pipes input to AI chat APIs and outputs responses. Perfect for integrating AI assistance into your shell workflows with support for multiple providers, streaming responses, and interactive mode.

## ✨ Features

- **Simple piping**: Read from stdin and send to AI
- **Multiple API providers**: Support for ch.at, ChatAnywhere, OpenAI, Claude, Ollama, and custom endpoints
- **Streaming responses**: Real-time response display with `--stream`
- **Interactive REPL mode**: Multi-turn conversations with `-i/--interactive`
- **Coder mode**: Generate clean code with `--coder`
- **File output**: Save responses directly to files with `--output`
- **Configuration system**: Save defaults in `~/.config/echomind/config.toml`
- **Advanced parameters**: Control temperature, max tokens, model selection
- **Progress indicators**: Visual feedback during API calls
- **Fast and async**: Optimized for performance with async I/O
- **Cross-platform**: Works on Linux, macOS, and Windows
- **User-friendly errors**: Clear, actionable error messages

## 📦 Installation

### From Source

1. Ensure you have Rust installed.
2. Clone or download this repository.
3. Run `cargo build --release` to build the executable.
4. The binary will be located at `target/release/echomind`.
5. Optionally, move it to a directory in your PATH, e.g., `~/.local/bin/echomind`.

### Automatic Install

```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | sh
```

This will automatically build and install echomind to `/usr/local/bin`.

### Arch Linux (AUR)

Manually build the package:

1. Clone this repo.
2. Use the provided `PKGBUILD` to build with `makepkg -si`.

### Pre-built Binaries

Download pre-built binaries from the [Releases](https://github.com/thepinak503/echomind/releases) page.

## 🚀 Usage

### Basic Usage

Pipe input to echomind from stdin:

```bash
echo "Hello, how are you?" | echomind
```

Use with other commands:

```bash
cat file.txt | echomind
git diff | echomind "Summarize these changes"
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
| `--api-key <KEY>` | | API key for provider |
| `--timeout <SECS>` | | Request timeout in seconds |
| `--verbose` | `-v` | Enable verbose output |
| `--init-config` | | Initialize default config file |
| `--show-config` | | Show config file location and contents |
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

# Debug error messages
./script.sh 2>&1 | echomind "What's wrong with this error?"

# Generate code from description
echo "Create a Rust function to parse JSON" | echomind -co parser.rs

# Review code
cat main.rs | echomind "Review this code for issues"

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

## 🚧 Roadmap

- [ ] Conversation history persistence
- [ ] Plugin system for custom providers
- [ ] Output formatting options (JSON, Markdown)
- [ ] Clipboard integration
- [ ] Voice input/output support
- [ ] Multi-model comparison mode

---

**Made with ❤️ using Rust**
