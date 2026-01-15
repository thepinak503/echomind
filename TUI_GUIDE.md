# Echomind TUI Usage Guide

## Overview
The Echomind TUI (Text User Interface) provides an interactive terminal-based chat interface with multiple AI providers, real-time streaming, and advanced features.

## Quick Start

### Launch the TUI
```bash
echomind --tui
```

### With Custom Settings
```bash
echomind --tui --provider openai --model gpt-4 --temperature 0.7 --stream
```

## Interface Layout

The TUI is divided into 4 main sections:

### 1. **Header (Top)**
- Shows current provider, model, temperature, and streaming status
- Displays total number of messages in the conversation
- Updates in real-time as settings change

### 2. **Chat History (Middle - Large)**
- Displays all messages in the conversation
- Shows timestamp, speaker (You/AI Provider/System), and message content
- Color-coded by message type:
  - **Blue (Bold)**: Your messages
  - **Green**: AI provider responses
  - **Yellow (Italic)**: System messages and commands
- Scrollable with arrow keys when focused on response

### 3. **Input Area (Below Chat)**
- Type your message here
- Shows current input mode (Ready/Processing/Complete)
- Displays helpful hints about command mode
- Cursor is visible in this area when active

### 4. **Footer (Bottom)**
- Displays status information
- Shows response time after completion
- Lists keyboard shortcuts

## Keyboard Controls

### General Navigation
| Key | Action |
|-----|--------|
| **Enter** | Send message or command |
| **Backspace** | Delete previous character |
| **↑/↓** | Navigate input history |
| **Esc** | Quit the application |
| **Ctrl+C** or **Ctrl+Q** | Force quit |
| **Tab** | Switch between input and response view |

### Settings Control
| Key | Action |
|-----|--------|
| **Ctrl+T** | Cycle temperature (0.1 → 0.5 → 1.0 → 1.8) |
| **Ctrl+S** | Toggle streaming mode on/off |
| **Ctrl+L** | Clear all messages and start fresh |
| **Ctrl+R** | Clear last response |

### Navigation
| Key | Action |
|-----|--------|
| **↑** (in Response) | Scroll chat up |
| **↓** (in Response) | Scroll chat down |
| **↑** (in Input) | Previous message in history |
| **↓** (in Input) | Next message in history |

## Commands

Commands start with `/` and provide various utilities:

### `/help` or `/h`
Display all available commands

### `/clear` or `/c`
Clear all messages and start a new conversation

### `/settings`
Display current configuration:
- Provider
- Model
- Temperature
- Max Tokens
- Stream status

### `/model <name>`
Change the AI model
```
/model gpt-4
/model claude-3-opus
```

### `/temp <value>`
Set temperature (0.0-2.0)
```
/temp 0.7
/temp 1.5
```

### `/export`
Save current chat to encrypted history file

### `/count`
Display total number of messages in conversation

## Features

### 1. **Multi-Provider Support**
- OpenAI (GPT-3.5, GPT-4)
- Claude (Anthropic)
- Ollama (Local models)
- Grok
- Mistral
- Cohere
- And more...

### 2. **Real-Time Streaming**
- Enable with `--stream` flag or `Ctrl+S` in TUI
- See responses appear word-by-word as they're generated
- Cancel streaming with Ctrl+C

### 3. **Message History**
- Auto-saved and encrypted locally
- Load previous conversations on startup
- Navigate history with ↑/↓ keys
- Search through history by scrolling

### 4. **Configurable Parameters**
- **Temperature**: Controls randomness (0=deterministic, 2=creative)
- **Max Tokens**: Limit response length
- **Top-P**: Nucleus sampling parameter
- **Top-K**: Top-K sampling parameter

### 5. **State Management**
The TUI tracks 3 states:
- **Input (Ready)**: Green border, ready to type
- **Processing (Yellow)**: Yellow border, AI is generating response
- **Response (Complete)**: Gray border, response received and displayed

### 6. **Encryption**
- All chat histories are encrypted with AES-256
- Stored in `~/.config/echomind/chat_history.enc`
- Automatic encryption/decryption on save/load

## Tips and Tricks

### 1. **Efficient Navigation**
- Use arrow keys to quickly navigate message history
- Hold Shift+PageUp/PageDown for faster scrolling
- Type `/help` anytime to see available commands

### 2. **Working with Responses**
- Long responses are wrapped automatically
- Use arrow keys to scroll through large responses
- Press Tab to switch focus between input and chat area

### 3. **Temperature Control**
- Start with 0.5-0.7 for balanced responses
- Use 0.1 for precise, factual responses
- Use 1.5+ for creative writing

### 4. **Multi-Turn Conversations**
- Messages accumulate in the conversation
- Context from previous messages is maintained
- Clear with `/clear` to start fresh session

### 5. **Batch Operations**
- Chain commands together
- `/clear` then type new message to reset context
- Use `/export` before experimenting

## Configuration

### Environment Variables
```bash
ECHOMIND_API_KEY=your_api_key ./echomind --tui
```

### Config File
Edit `~/.config/echomind/config.toml`:
```toml
[api]
provider = "openai"
model = "gpt-4"
api_key = "your-key-here"

[defaults]
temperature = 0.7
max_tokens = 2000
stream = true
```

## Troubleshooting

### "No internet connection"
- Check your network connection
- Verify firewall allows outgoing connections
- Use `ping 8.8.8.8` to test connectivity

### "API Key Invalid"
- Set `ECHOMIND_API_KEY` environment variable
- Or add to config file at `~/.config/echomind/config.toml`
- Ensure key is not expired or revoked

### "Slow Responses"
- Check internet connection
- Try increasing timeout: `--timeout 60`
- Use more specific models if available

### "Terminal Display Issues"
- Try resizing your terminal window
- Clear screen with `clear` before launching
- Ensure terminal supports 256 colors

## Performance Notes

- **Memory**: Efficient streaming reduces memory usage
- **CPU**: Minimal CPU usage during API calls
- **Network**: Uses async I/O for non-blocking operations
- **Storage**: Encrypted history files are compact

## Security

- **Encryption**: AES-256-GCM for local chat history
- **API Keys**: Never logged or displayed
- **Network**: Supports HTTPS only
- **Privacy**: Chat history stored locally, not sent to servers

## Keyboard Shortcuts Cheat Sheet

```
┌─────────────────────────────┐
│ BASIC SHORTCUTS             │
├─────────────────────────────┤
│ Ctrl+C/Ctrl+Q  │ Quit       │
│ Ctrl+T         │ Temp       │
│ Ctrl+S         │ Stream     │
│ Ctrl+L         │ Clear All  │
│ ↑/↓            │ History    │
│ Tab            │ Switch     │
│ /help          │ Commands   │
└─────────────────────────────┘
```

## Advanced Usage

### Piping to TUI
While the TUI is interactive, you can prepare input with:
```bash
# Not directly piped, but useful for batch:
echo "greeting" | echomind  # Non-TUI mode
echomind --tui              # Interactive mode
```

### Combining with Other Tools
```bash
# Get help about TUI features
echomind --help | grep -i tui

# Run with verbose output for debugging
echomind --tui --verbose
```

## Getting Help

### In-App Help
- Type `/help` in the TUI
- Press `?` for keyboard shortcuts (if implemented)
- Check status bar for current mode

### External Resources
- Run `echomind --help` for all options
- Check `README.md` for general information
- Visit GitHub issues for bug reports

## Version Information
Echomind TUI is part of Echomind v0.3.1+

Last Updated: January 2026
