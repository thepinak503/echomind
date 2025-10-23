# echomind

A simple Linux executable written in Rust that reads input from stdin (typically piped from another command) and sends it as a message to a chat API. The response from the API is printed to stdout.

## Installation

### From Source

1. Ensure you have Rust installed.
2. Clone or download this repository.
3. Run `cargo build --release` to build the executable.
4. The binary will be located at `target/release/echomind`.
5. Optionally, move it to a directory in your PATH, e.g., `~/.local/bin/echomind`.

### Arch Linux (AUR)

For Arch Linux users, you can install via AUR:

```bash
yay -S echomind
```

Or manually build the package:

1. Clone this repo.
2. Use the provided `PKGBUILD` to build with `makepkg -si`.

## Usage

Pipe input to the executable:

```bash
echo "Hello, how are you?" | echomind
```

Or from another command:

```bash
some_command | echomind
```

## Options

- `--help`, `-h`: Display help information.

## API

This tool uses the chat API at `https://ch.at/v1/chat/completions`. It sends a JSON payload with the input as a user message and prints the assistant's response.

## Dependencies

- reqwest for HTTP requests
- serde for JSON serialization/deserialization
- tokio for async runtime
- clap for command-line argument parsing

## License

[Add your license here, e.g., MIT]