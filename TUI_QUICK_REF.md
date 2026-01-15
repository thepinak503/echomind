# Echomind TUI - Quick Reference Card

## Launch

```bash
echomind --tui
echomind --tui --provider openai --model gpt-4
echomind --tui --stream --temperature 0.7
```

## Keyboard Shortcuts

### Core Controls
| Shortcut | Action |
|----------|--------|
| **Enter** | Send message / Change state |
| **Esc** / **Ctrl+Q** / **Ctrl+C** | Quit |
| **Backspace** | Delete character |
| **Ctrl+T** | Cycle temperature (0.1 → 0.5 → 1.0 → 1.8) |
| **Ctrl+S** | Toggle streaming |
| **Ctrl+L** | Clear all messages |
| **Ctrl+R** | Clear last response |

### Navigation
| Shortcut | Action |
|----------|--------|
| **↑** (Input) | Previous message in history |
| **↓** (Input) | Next message in history |
| **↑** (Response) | Scroll chat up |
| **↓** (Response) | Scroll chat down |
| **Tab** | Switch between input and response view |

## Commands (Type and Enter)

### Help & Info
```
/help           Display all available commands
/count          Show total number of messages
/settings       Display current settings
```

### Management
```
/clear          Clear entire conversation
/export         Save chat to encrypted file
```

### Configuration
```
/model gpt-4    Change to specific model
/temp 0.7       Set temperature to 0.7
```

## Interface Elements

```
┌─ HEADER (Status Info) ─────────────────────┐
│ Provider: openai | Model: gpt-4 | Temp: 0.7│
├─ CHAT AREA (Messages) ─────────────────────┤
│ [14:23:45] You: Hello                      │
│ [14:23:47] openai: Hi there!               │
│                                             │
├─ INPUT AREA ──────────────────────────────┤
│ > Your message here                        │
│ (type /help for commands)                  │
├─ FOOTER (Help & Status) ───────────────────┤
│ Ready | ^T: temp, ^S: stream, ^Q: quit    │
└─────────────────────────────────────────────┘
```

## Message Colors

- 🔵 **Blue & Bold** = Your message
- 🟢 **Green** = AI response
- 🟡 **Yellow & Italic** = System message

## Temperature Values

| Value | Behavior |
|-------|----------|
| 0.1 | Very precise, factual |
| 0.5 | Balanced, slightly creative |
| 1.0 | Natural, moderate creativity |
| 1.5 | Creative, imaginative |
| 1.8+ | Very creative, experimental |

## Common Workflows

### Start a New Conversation
```
1. Launch: echomind --tui
2. Type message
3. Press Enter
4. Wait for response
```

### Change Temperature During Chat
```
1. Press Ctrl+T repeatedly to cycle through: 0.1 → 0.5 → 1.0 → 1.8
2. Or type: /temp 0.7
3. Continue chatting
```

### Switch Models
```
1. Type: /model claude-3-opus
2. Continue chatting with new model
3. (Requires API key for that provider)
```

### Clear and Start Fresh
```
1. Type: /clear
2. Or press: Ctrl+L
3. Type new message
```

### View Current Settings
```
1. Type: /settings
2. Shows all current configuration
```

## Configuration File

Location: `~/.config/echomind/config.toml`

```toml
[api]
provider = "openai"
model = "gpt-4"
api_key = "sk-..."
timeout = 30

[defaults]
temperature = 0.7
max_tokens = 2000
stream = true
```

## Environment Variables

```bash
export ECHOMIND_API_KEY="your-key-here"
echomind --tui
```

## State Indicators

- 🟢 **Green Input Box** = Ready to type
- 🟡 **Yellow Input Box** = Processing response
- ⚪ **Gray Input Box** = Response complete
- ◐ ◓ ◑ ◒ = Processing spinner animation

## Common Issues

| Issue | Solution |
|-------|----------|
| No API key | Set `ECHOMIND_API_KEY` or use config file |
| Slow response | Check internet, increase timeout: `--timeout 60` |
| Display issues | Resize terminal, ensure 256-color support |
| Messages not saving | Check `~/.config/echomind/` directory exists |

## Status Bar Info

During Input:
```
Ready | ↑↓: history | Ctrl+Q: quit
```

During Processing:
```
◐ Processing... (Ctrl+C to cancel)
```

After Response:
```
Response received in 2s | ↑↓: scroll | ^Q: quit
```

## Pro Tips

1. **Quick Settings Loop**: Hold Ctrl+T to find perfect temperature
2. **History Search**: Use ↑/↓ to quickly find previous messages
3. **Parallel Sessions**: Open multiple terminals for separate conversations
4. **Command Chaining**: Type multiple commands in sequence
5. **Export Before Experimenting**: Use `/export` before testing new settings

## Advanced Shortcuts

- **Ctrl+U** (if supported): Clear input line
- **Ctrl+W** (if supported): Delete word backward
- **Home/End** (if supported): Move to line start/end

## Full Help

For complete documentation, see:
- `TUI_README.md` - User guide
- `TUI_GUIDE.md` - Detailed guide with examples
- `TUI_TECHNICAL.md` - Developer documentation

---

**Version**: Echomind 0.3.1+
**Last Updated**: January 2026
Print this card and keep it handy!
