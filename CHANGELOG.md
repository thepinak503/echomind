# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2024-10-30

### 🎉 Major Release - Comprehensive Enhancements

This release represents a complete overhaul of echomind with numerous new features, improved architecture, and better user experience.

### Added

#### Core Features
- **Multiple API Provider Support**: Now supports chat (ch.at), ChatAnywhere, OpenAI, Claude, Ollama, and custom endpoints
- **Interactive REPL Mode**: Multi-turn conversations with `-i/--interactive` flag
- **Streaming Responses**: Real-time response display with `--stream` flag
- **Configuration System**: TOML-based config file at `~/.config/echomind/config.toml`
- **Progress Indicators**: Visual feedback during API calls with spinners

#### CLI Enhancements
- `--provider` / `-p`: Select API provider
- `--model` / `-m`: Choose specific model
- `--temperature` / `-t`: Control response randomness (0.0-2.0)
- `--max-tokens`: Limit response length
- `--system` / `-s`: Custom system prompts
- `--stream`: Enable streaming mode
- `--interactive` / `-i`: Interactive REPL mode
- `--api-key`: Specify API key (also via ECHOMIND_API_KEY env var)
- `--timeout`: Configure request timeout
- `--verbose` / `-v`: Enable debug output
- `--init-config`: Create default configuration file
- `--show-config`: Display config file location and contents

#### Developer Features
- **Modular Architecture**: Code organized into `api.rs`, `cli.rs`, `config.rs`, `error.rs`, `repl.rs`
- **Custom Error Types**: User-friendly error messages with specific context
- **Test Suite**: Comprehensive unit and integration tests
- **CI/CD Pipeline**: GitHub Actions for testing, linting, releases, and security audits

#### Documentation
- **Enhanced README**: Comprehensive usage examples and feature documentation
- **CONTRIBUTING.md**: Guidelines for contributors
- **Man Page**: Updated with all new features
- **Example Config**: `config.example.toml` with detailed comments

### Changed
- **Better Error Handling**: Clear, actionable error messages instead of generic errors
- **Improved Help Display**: Shows usage info when run without arguments
- **Enhanced Coder Mode**: Automatically removes markdown code fences from output
- **Color-coded Output**: Better visual feedback with colored terminal output

### Technical Improvements
- Async/await throughout for better performance
- Better input handling (terminal detection)
- Progress indicators for long-running requests
- Proper timeout handling with user feedback
- API key management via config, environment, or CLI

### Dependencies Added
- `toml` - Configuration file parsing
- `dirs` - Cross-platform config directories
- `thiserror` - Better error handling
- `anyhow` - Error context
- `indicatif` - Progress bars
- `colored` - Terminal colors
- `rustyline` - Interactive REPL
- `futures` - Async streaming
- `eventsource-stream` - SSE support

### Testing
- 10 unit tests for API and configuration
- Mock-based testing infrastructure
- CI/CD with automated testing on Linux, macOS, and Windows

### Documentation
- Complete rewrite of README with examples
- Contributing guidelines
- Updated man page
- Configuration examples
- GitHub workflows for CI/CD

## [0.2.0] - Previous Release

### Added
- Basic coder mode
- File output support
- Combined `--co` flag

## [0.1.0] - Initial Release

### Added
- Basic stdin to AI API piping
- Simple error handling
- Basic CLI interface

