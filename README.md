# echomind

A lightweight, fast command-line tool written in Rust that pipes input to an AI chat API and outputs the response. Perfect for integrating AI assistance into your shell workflows.

## Features

- **Simple piping**: Read from stdin and send to AI
- **Coder mode**: Generate clean code with `--coder`
- **File output**: Save responses directly to files with `--output`
- **Fast and async**: Optimized for performance with async I/O
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

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

## Usage

Pipe input to echomind from stdin:

```bash
echo "Hello, how are you?" | echomind
```

Use with other commands:

```bash
cat file.txt | echomind
```

Generate code and save to file:

```bash
echo "write a Python function to calculate factorial" | echomind --coder --output factorial.py
```

Short options work too:

```bash
echo "explain this code" | cat code.js | echomind -c -o explanation.txt
```

Combined short options:

```bash
echo "optimize this SQL query" | echomind -co optimized.sql
```

## Options

- `-c`, `--coder`: Enable coder mode. Adds a system prompt to generate clean, runnable code without explanations.
- `-o`, `--output <FILE>`: Save the response to a file instead of printing to stdout. In coder mode, empty lines are filtered.
- `-h`, `--help`: Display help information and exit.
- `-V`, `--version`: Display version information and exit.

## How It Works

echomind reads all input from stdin, constructs a chat completion request, and sends it to the ch.at API. The API's response is then processed and outputted.

In normal mode, your input is sent as a user message. In coder mode (`--coder`), a system prompt is prepended to instruct the AI to output only raw code.

## API

This tool uses the chat API at `https://ch.at/v1/chat/completions`. It sends a JSON payload with the messages and receives the assistant's response. Requests timeout after 30 seconds to prevent hanging.

## Dependencies

- reqwest for HTTP requests
- serde for JSON serialization/deserialization
- tokio for async runtime
- clap for command-line argument parsing

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Credits

Thanks to the ch.at API for the inspiration behind this tool. This project leverages the power of AI chat completions to make command-line interactions more intelligent and fun. Special shoutout to the open-source community for providing the libraries that make Rust development a breeze.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.