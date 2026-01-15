# Echomind TUI - Technical Documentation

## Architecture Overview

The Echomind TUI is built with:
- **Framework**: Ratatui (terminal UI framework)
- **Async Runtime**: Tokio
- **Terminal Backend**: Crossterm
- **Serialization**: Serde JSON
- **Encryption**: Ring (AES-256-GCM)

## Code Structure

### Main Components

#### 1. `App` Struct
**Location**: `src/tui.rs` (lines 34-71)

```rust
pub struct App {
    state: AppState,           // Current UI state
    input: String,             // User input buffer
    messages: Vec<AppMessage>, // Chat history
    provider: Provider,        // AI provider
    model: String,             // Model name
    temperature: f32,          // Generation temperature
    max_tokens: Option<u32>,   // Token limit
    top_p: Option<f32>,        // Nucleus sampling
    top_k: Option<u32>,        // Top-K sampling
    stream: bool,              // Streaming enabled
    history: Vec<String>,      // Command history
    history_index: Option<usize>, // History navigation
    config: Config,            // Configuration
    args: Args,                // CLI arguments
    scroll_offset: u16,        // Chat area scroll position
    processing_spinner: usize, // Spinner animation index
    last_response_time: Option<Instant>, // Response timing
}
```

#### 2. `AppMessage` Struct
**Location**: `src/tui.rs` (lines 24-28)

```rust
pub struct AppMessage {
    role: String,      // "You", provider name, or "System"
    content: String,   // Message text
    timestamp: String, // HH:MM:SS format
}
```

Derives: `Debug`, `Clone`, `Serialize`, `Deserialize`

#### 3. `AppState` Enum
**Location**: `src/tui.rs` (lines 20-24)

```rust
enum AppState {
    Input,       // Waiting for user input
    Processing,  // AI is generating response
    Response,    // Response complete, awaiting next input
}
```

### Key Functions

#### `run_app`
**Location**: `src/tui.rs` (lines 173-328)

Main event loop. Handles:
- Terminal rendering
- Event processing (keyboard, etc.)
- Response channel monitoring
- State transitions

**Event Handling**:
- `KeyCode::Enter`: Send message or change state
- `KeyCode::Char(c)`: Add character to input
- `KeyCode::Backspace`: Delete character
- `KeyCode::Up/Down`: Navigate history or scroll
- `KeyCode::Esc`: Quit application
- Control key combinations for settings

**Asynchronous Processing**:
- Creates tokio task for API calls
- Uses `mpsc::unbounded_channel` for response streaming
- Non-blocking UI during API calls

#### `process_query`
**Location**: `src/tui.rs` (lines 380-417)

Handles API calls asynchronously:
```rust
async fn process_query(
    input: String,
    provider: Provider,
    model: String,
    temperature: f32,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
    stream: bool,
    config: Config,
    args: Args,
    tx: mpsc::UnboundedSender<String>,
) -> Result<()>
```

Features:
- Creates `ApiClient` with configuration
- Builds `ChatRequest` with parameters
- Supports streaming and non-streaming modes
- Sends response chunks through channel

#### `ui` Function
**Location**: `src/tui.rs` (lines 419-558)

Renders the terminal interface:

**Layout Constraints**:
1. Header (3 lines)
2. Chat area (flexible, minimum 4 lines)
3. Input area (4 lines)
4. Footer (2 lines)

**Color Scheme**:
- Header: Dark gray background, white text
- Chat: Cyan borders, white text
- Messages: Role-colored with styles
- Input: Status-dependent color
- Footer: Gray italic text

### Data Flow

```
User Input
    ↓
Event Handler
    ↓
├─ Regular Message → Add to history, create task
├─ Command → Process command, update state
├─ Control Key → Modify settings
└─ Navigation → Scroll or navigate history
    ↓
Async Processing (if message)
    ├─ API call
    └─ Stream response through channel
    ↓
Channel Receiver
    ↓
Update App State
    ↓
Render UI
```

## Message Encryption/Decryption

### Encryption
**Location**: `src/tui.rs` (lines 143-158)

Uses AES-256-GCM:
```rust
fn encrypt(data: &[u8]) -> Result<Vec<u8>> {
    // Fixed key (in production, derive from user password)
    let key_bytes = b"01234567890123456789012345678901";
    // Fixed nonce (12 bytes of zeros)
    let nonce_bytes = [0u8; 12];
    // Encrypt and append auth tag
}
```

**Important Notes**:
- Uses fixed key and nonce (consider randomizing)
- Stores in `~/.config/echomind/chat_history.enc`
- All messages serialized as JSON before encryption

### Decryption
**Location**: `src/tui.rs` (lines 160-173)

Reverse of encryption:
```rust
fn decrypt(data: &[u8]) -> Result<Vec<u8>> {
    // Same key and nonce as encryption
    // Decrypts and verifies authentication tag
}
```

## Command Processing

**Location**: `src/tui.rs` (lines 104-142)

Commands implement prefix-based routing:
```rust
fn process_command(&mut self) -> bool {
    match parts.get(0).map(|&s| s) {
        Some("/help") => { /* ... */ }
        Some("/clear") => { /* ... */ }
        Some("/settings") => { /* ... */ }
        Some("/model") => { /* ... */ }
        Some("/temp") => { /* ... */ }
        Some("/export") => { /* ... */ }
        Some("/count") => { /* ... */ }
        _ => { /* Unknown command */ }
    }
}
```

Each command:
1. Parses arguments
2. Validates input
3. Updates state
4. Optionally adds system message
5. Clears input

## UI Rendering Details

### Header Rendering
- Displays app name and status
- Shows all current settings
- Updates message counter
- Uses color coding for streaming status

### Chat Area Rendering
- Iterates through all messages
- Applies role-specific styling
- Supports text wrapping
- Implements scroll position
- Color-codes by message type:
  - "You": Blue + Bold
  - "System": Yellow + Italic
  - Others: Green

### Input Area Rendering
- Shows current input buffer
- Displays status title (Ready/Processing/Complete)
- Changes border color based on state
- Shows helpful hints for user

### Footer Rendering
- Spinner animation during processing
- Response timing information
- Keyboard shortcut reminders

## State Machine

```
Initial State: Input

Input State:
├─ User types → Input buffer updates
├─ User presses Enter (message) → Processing state
├─ User presses Enter (command) → Command processed, stay in Input
├─ User presses Esc → Exit
└─ Ctrl+T/S/L → Modify settings, stay in Input

Processing State:
├─ API response received → Response state
├─ Response chunks → Update last message
└─ Spinner animation updates

Response State:
├─ User presses Enter → Input state
├─ User navigates → Scroll response
└─ Timeout (optional) → Return to Input
```

## Async Patterns

### Message Channel
Uses `mpsc::unbounded_channel::<String>()` for streaming:

1. **Sender** created in event loop
2. **Cloned** to spawned tokio task
3. **Used** by API client to send chunks
4. **Received** back in event loop
5. **Processed** into message updates

### Non-blocking UI
- Event handling uses 100ms poll timeout
- Prevents blocking on `event::read()`
- Allows spinner animation between events
- Respects rapid user input

## Performance Considerations

### Memory
- Message vector grows with conversation
- Consider pagination for very long conversations
- Scroll position tracked efficiently

### CPU
- Spinner animation: 1 update per event loop (~100ms)
- Rendering: Only on event or channel update
- No busy-waiting

### Network
- Async/await prevents blocking
- Streaming chunks reduce latency perception
- Response appears real-time

## Extension Points

### Adding New Commands
1. Add case to `process_command()` match
2. Implement command logic
3. Add system message for feedback
4. Clear input

### Adding New Settings
1. Add field to `App` struct
2. Add UI display in header or settings command
3. Add keyboard shortcut (if simple toggle)
4. Consider persisting to config

### Custom Styling
1. Modify color schemes in `ui()` function
2. Change `Style::default()` values
3. Use different `Span::styled()` combinations
4. Consider theme configuration

### Response Processing
1. Modify message update logic in event loop
2. Add parsing or formatting
3. Could implement syntax highlighting
4. Could add emoji or markdown rendering

## Testing

### Manual Testing Checklist
- [ ] Launch with `--tui` flag
- [ ] Type and send message
- [ ] Test each keyboard shortcut
- [ ] Test each command
- [ ] Verify streaming works
- [ ] Check history encryption/loading
- [ ] Test multiple providers (if configured)
- [ ] Test very long messages
- [ ] Test rapid keypresses

### Unit Testing Opportunities
- Encryption/decryption functions
- Command parsing logic
- Message formatting
- State transitions

## Known Limitations

1. **Fixed Encryption Keys**: Uses hardcoded key, should be user-derived
2. **Single Provider per Session**: Can't switch between providers mid-session
3. **No Message Deletion**: Can only clear all messages
4. **Limited Search**: No search through message history
5. **No Formatting**: No markdown rendering in responses
6. **No Image Support**: Text-only in TUI mode
7. **Fixed Window Layout**: Can't customize pane sizes

## Future Enhancements

1. **User Preferences**
   - Custom color themes
   - Configurable key bindings
   - Layout customization

2. **Advanced Features**
   - Message search and filtering
   - Export to different formats (JSON, CSV, PDF)
   - Conversation bookmarking
   - Message editing/deletion

3. **UI Improvements**
   - Markdown rendering
   - Code syntax highlighting
   - Rich text formatting
   - Emoji/Unicode support

4. **Performance**
   - Message pagination
   - Lazy loading of history
   - Optimized rendering

5. **Usability**
   - Help modal
   - Command autocomplete
   - Keyboard shortcut customization
   - Multi-session management

## Dependencies

Key crates:
- `ratatui`: TUI framework
- `crossterm`: Terminal backend
- `tokio`: Async runtime
- `ring`: Cryptography
- `serde_json`: Serialization
- `chrono`: Timestamps
- `dirs`: Directory handling

Version compatibility: Rust 1.70+

## Code Quality

### Current
- Warnings cleaned up
- Error handling with `Result` type
- Async/await patterns used correctly
- No unsafe code blocks

### Areas for Improvement
- Could use more comprehensive error messages
- Consider custom error types vs generic `EchomindError`
- Could extract ui() into sub-functions
- Process_command() could use more validation

## Integration with Main App

The TUI is launched from `src/main.rs`:

```rust
if args.tui {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let app = App::new(config, args.clone());
    let res = crate::tui::run_app(&mut terminal, app).await;
    // ... cleanup
}
```

Uses same configuration as CLI mode, maintains compatibility.

## References

- Ratatui Documentation: https://docs.rs/ratatui/
- Crossterm Documentation: https://docs.rs/crossterm/
- Tokio Async Guide: https://tokio.rs/
- Ring Crypto: https://briansmith.org/rustdoc/ring/

---

Last Updated: January 2026
For contribution guidelines, see CONTRIBUTING.md
