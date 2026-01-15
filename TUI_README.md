# Echomind TUI - Terminal User Interface

## What is the TUI?

The Echomind TUI (Text User Interface) is an interactive terminal-based chat application that provides a full-featured AI chat interface in your terminal. It supports multiple AI providers, real-time streaming, message history, and advanced configuration options.

## Quick Start

```bash
# Launch TUI with default settings
echomind --tui

# With OpenAI's GPT-4
echomind --tui --provider openai --model gpt-4

# With streaming enabled
echomind --tui --stream

# With custom temperature
echomind --tui --temperature 0.7
```

## Features

✅ **Multi-Provider Support**
- OpenAI (GPT-3.5, GPT-4, etc.)
- Anthropic Claude
- Ollama (local models)
- Grok, Mistral, Cohere, and more

✅ **Real-Time Streaming**
- See responses appear word-by-word as they're generated
- Toggle streaming with Ctrl+S

✅ **Message History**
- Auto-save and encrypt conversations locally
- Load previous chats automatically
- Navigate history with ↑/↓ keys

✅ **Configurable Settings**
- Adjust temperature on-the-fly (Ctrl+T)
- Toggle streaming (Ctrl+S)
- Change model, API key, and other parameters
- Clear conversation (Ctrl+L)

✅ **Command System**
- `/help` - Show available commands
- `/clear` - Clear all messages
- `/settings` - View configuration
- `/model <name>` - Change AI model
- `/temp <value>` - Set temperature
- `/export` - Save chat to file
- `/count` - Show message count

✅ **Encrypted Storage**
- All chat histories encrypted with AES-256-GCM
- Stored in `~/.config/echomind/chat_history.enc`
- Automatic save/load

✅ **Intuitive Interface**
- Clean layout with chat history, input, and status
- Color-coded messages by role
- Real-time status indicators
- Helpful keyboard shortcuts

## Key Keyboard Shortcuts

| Key | Action |
|-----|--------|
| **Enter** | Send message |
| **Ctrl+C** or **Esc** | Quit |
| **Ctrl+T** | Cycle temperature |
| **Ctrl+S** | Toggle streaming |
| **Ctrl+L** | Clear all messages |
| **↑/↓** | Navigate message history |
| **Tab** | Switch focus (input/chat) |

## Interface Sections

```
┌─────────────────────────────────────────┐
│ Echomind TUI │ Provider │ Model │ Temp  │  <- Header (Status)
├─────────────────────────────────────────┤
│                                         │
│          Chat History Display           │
│   Shows all messages with timestamps    │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│ ┌───────────────────────────────────┐   │
│ │ Input Box (Type your message)      │   │  <- Input Area
│ │ [Ready/Processing/Complete]       │   │
│ └───────────────────────────────────┘   │
├─────────────────────────────────────────┤
│ Tips: Type /help for commands           │  <- Footer (Help)
└─────────────────────────────────────────┘
```

## Configuration

### Command Line Flags

```bash
echomind --tui [OPTIONS]

OPTIONS:
  -p, --provider <NAME>      AI provider (openai, claude, ollama, etc.)
  -m, --model <MODEL>        Model name (gpt-4, claude-3-opus, etc.)
  -t, --temperature <NUM>    Temperature 0.0-2.0 (default: 0.7)
  --stream                   Enable streaming responses
  --api-key <KEY>            API key (or use ECHOMIND_API_KEY env var)
  --timeout <SECS>           Request timeout in seconds
```

### Config File

Edit `~/.config/echomind/config.toml`:

```toml
[api]
provider = "openai"
model = "gpt-4"
api_key = "sk-..."  # or use ECHOMIND_API_KEY env var
timeout = 30

[defaults]
temperature = 0.7
max_tokens = 2000
stream = true
top_p = 0.9
```

## Example Session

```
$ echomind --tui

┌─── Echomind TUI ───────────────────────┐
│ Provider: openai | Model: gpt-4        │
│ Temperature: 0.70 | Stream: On         │
├────────────────────────────────────────┤
│                                        │
│  [14:23:45] You: Hello!               │
│  [14:23:47] openai: Hi there! How can │
│  I help you today?                     │
│                                        │
├────────────────────────────────────────┤
│ > Explain quantum computing            │
│ (type /help for commands)              │
├────────────────────────────────────────┤
│ Ready | ↑↓: history | Ctrl+Q: quit    │
└────────────────────────────────────────┘
```

## Troubleshooting

### "API key not found"
```bash
export ECHOMIND_API_KEY="your-key-here"
echomind --tui
```

### "No internet connection"
- Check your network
- Verify firewall allows outgoing connections
- Try: `ping 8.8.8.8`

### "Slow responses"
- Try increasing timeout: `--timeout 60`
- Check your internet connection
- Try a different model

### Terminal display issues
- Resize your terminal window
- Try clearing screen first: `clear`
- Ensure 256-color support: `echo $TERM`

## Tips & Tricks

1. **Quick temperature cycling**: Press Ctrl+T repeatedly to cycle through common temperature values
2. **History navigation**: Use ↑/↓ to quickly select previous messages
3. **Long responses**: Use Tab to switch focus, then arrow keys to scroll
4. **Commands anytime**: Type `/help` to see available commands
5. **Export chats**: Use `/export` before experimenting with settings

## Advanced Usage

### Using Custom Configuration

```bash
# Create custom config
cp config.example.toml ~/.config/echomind/config.toml
# Edit and set your preferences
nano ~/.config/echomind/config.toml

# Launch TUI (reads config automatically)
echomind --tui
```

### Multiple Sessions

Each TUI session maintains its own message history:
```bash
echomind --tui  # Session 1
# In another terminal:
echomind --tui  # Session 2 (loads same history by default)
```

### Combining with Pipes

While TUI is interactive, you can check settings:
```bash
echomind --help | grep -i tui
```

## Performance

- **Lightweight**: Minimal CPU and memory usage
- **Responsive**: Non-blocking async I/O
- **Efficient**: Streaming reduces perceived latency
- **Fast startup**: Direct terminal mode, no GUI overhead

## Security

- **Local encryption**: Chat history encrypted with AES-256-GCM
- **No telemetry**: All processing stays local unless API called
- **API only**: Network calls only to configured AI providers
- **API key protection**: Keys never logged or displayed

## Limitations

- Text-only mode (no image support in TUI)
- Single conversation per session
- No message deletion or editing
- Fixed terminal layout

## Getting Help

- Type `/help` in the TUI
- Run `echomind --help` for general options
- Check `TUI_GUIDE.md` for detailed usage
- See `TUI_TECHNICAL.md` for developer documentation

## Contributing

Want to improve the TUI? Check [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Common improvement areas:
- New commands
- Better UI layouts
- Additional color schemes
- Performance optimizations
- Bug fixes

## License

Same as Echomind - See LICENSE file for details

---

**Version**: 0.3.2+
**Last Updated**: January 2026

For more information, visit the [Echomind GitHub Repository](https://github.com/pinak/echomind)
