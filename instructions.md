# Echomind Project Instructions for AI LLM CLI Agents

## Overview
Echomind is a powerful, lightweight command-line tool written in Rust that pipes input to AI chat APIs and outputs responses. It supports multiple providers, streaming, interactive mode, and more. This document provides complete context and instructions for AI agents to maintain, build, install, and enhance the project.

## Project Structure
```
echomind/
├── src/                    # Rust source code
├── tests/                  # Test files
├── .github/workflows/      # CI/CD workflows
├── debian/                 # Debian packaging
├── Cargo.toml              # Rust dependencies
├── Cargo.lock              # Dependency lock
├── install.ps1             # Windows PowerShell installer
├── install.sh              # Full Unix installer
├── curl-install.sh         # One-liner curl installer (Linux)
├── echomind-linux-x86_64   # Pre-built Linux x86_64 binary
├── echomind-linux-x86_64.gz # Compressed Linux binary (1.7MB)
├── docs/                      # Documentation files
│   ├── README.md               # Main documentation
│   ├── CHANGELOG.md            # Version history
│   ├── RELEASE_NOTES.md        # Release notes
│   ├── LICENSE                 # License
│   ├── config.example.toml    # Example configuration
│   └── completions/            # Shell completions
│       ├── echomind.bash       # Bash
│       ├── echomind.fish       # Fish
│       ├── _echomind           # Zsh
│       └── _echomind.ps1       # PowerShell
│       ├── echomind.bash
│       ├── echomind.fish
│       ├── _echomind
│       └── _echomind.ps1
├── PKGBUILD                # Arch Linux package
├── .SRCINFO                # Arch package info
├── echomind.1              # Man page
├── CONTRIBUTING.md         # Contribution guidelines
└── instructions.md         # This file
```

## Build Process
### Prerequisites
- Rust 1.70+ (`cargo`, `rustc`)
- OpenSSL development libraries
- pkg-config
- For Windows: MSVC compiler (Visual Studio Build Tools)
- For Linux/macOS: Standard development tools (gcc, make, clang)

### Build Commands
```bash
# Clone repository
git clone https://github.com/thepinak503/echomind.git
cd echomind

# Build in release mode
cargo build --release

# Binary location: target/release/echomind (or .exe on Windows)
```

### Platform-Specific Compilation
#### Linux
- **Supported Distros**: Ubuntu 20.04+, Debian 11+, Arch, Fedora 35+, CentOS 8+, openSUSE 15.3+
- **Command**: `cargo build --release`
- **Output**: `target/release/echomind`
- **Notes**: Ensure development tools installed (build-essential on Debian/Ubuntu)

#### macOS
- **Supported Versions**: 10.15+ (Catalina+), Intel x86_64 and Apple Silicon ARM64
- **Command**: `cargo build --release`
- **Output**: `target/release/echomind`
- **Notes**: Xcode Command Line Tools required; native ARM64 support

#### Windows
- **Supported Versions**: Windows 10 1809+, Windows 11, Windows Server 2019+
- **Prerequisites**: MSVC compiler (install Visual Studio Build Tools from https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- **Command**: `cargo build --release`
- **Output**: `target/release/echomind.exe`
- **Fallback**: If MSVC unavailable, use pre-built `echomind.exe` from repo root
- **Notes**: Run in Developer Command Prompt or use install.ps1 which handles setup

## Installation Process

### Quick Install (One-liner)
```bash
# Linux x86_64 - Fastest method
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/curl-install.sh | bash

# Or directly download and install
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/echomind-linux-x86_64.gz | gunzip > /tmp/echomind && sudo mv /tmp/echomind /usr/bin/echomind && sudo chmod +x /usr/bin/echomind
```

### Automated Installers
- **Windows**: `install.ps1` - Installs Rust if needed, builds or falls back to repo exe, copies to user bin and optionally System32
- **Unix**: `install.sh` - Downloads and installs pre-built binaries
- **curl-install.sh**: One-liner curl script for Linux x86_64

### Pre-built Binaries in Repo
- `echomind-linux-x86_64` - Linux x86_64 binary (uncompressed)
- `echomind-linux-x86_64.gz` - Linux x86_64 binary (gzip compressed, 1.7MB)
- These can be curled directly from GitHub raw URL

### Manual Installation
1. Build the project
2. Copy binary to PATH (e.g., `~/.local/bin/echomind`)
3. Initialize config: `echomind --init-config`
4. Optionally copy `docs/config.example.toml` to config location

### Handling Installation Failures
If installation fails:
1. **Check Prerequisites**: Ensure Rust and compilers are installed
2. **Fallback to Pre-built**: Use `echomind.exe` from repo root
3. **System32 Copy**: Attempt copy to `C:\Windows\System32` (requires admin)
4. **PATH Setup**: Add installation directory to user PATH
5. **Iterate on Errors**: 
   - If build fails, modify code to fix compilation issues
   - Update dependencies in Cargo.toml
   - Add error handling in install scripts
   - Ask user for new suggestions if stuck

## Code Changes and Features
### Core Guidelines
- Follow Rust best practices
- Use async I/O for performance
- Support multiple AI providers (OpenAI, Claude, Gemini, etc.)
- Maintain CLI interface with piping support
- Add streaming responses and interactive mode

### Adding New Features
1. **Plan**: Analyze requirements and existing code
2. **Implement**: Add to src/, update Cargo.toml if needed
3. **Test**: Run `cargo test`, add unit tests
4. **Document**: Update docs/README.md, docs/CHANGELOG.md
5. **Commit**: Use descriptive messages, push to main branch

### User-Initiated Changes
When user requests changes:
1. **Understand Request**: Parse user instructions clearly
2. **Locate Code**: Find relevant files in src/
3. **Modify**: Use edit tool for precise changes
4. **Build/Test**: Ensure changes compile and work
5. **Iterate**: If issues, fix and re-test
6. **Commit/Push**: Stage, commit with clear message, push

### Common Change Patterns
- **New Provider**: Add to src/providers/, update CLI args
- **New Feature**: Implement in src/main.rs, add clap args
- **Bug Fix**: Locate issue, apply minimal fix, test
- **Config**: Update docs/config.example.toml, add parsing

## Error Handling and Iteration
### Build Failures
- **MSVC Missing**: Warn, fallback to pre-built exe
- **Dependency Issues**: Update Cargo.lock, check versions
- **Compilation Errors**: Fix syntax, add missing imports
- **Linker Errors**: Check system libraries, update build scripts
- **Missing Module Files**: Comment out pub mod declarations for non-existent files in src/features/mod.rs
- **Undefined CLI Fields**: Comment out usage of disabled CLI arguments in main.rs
- **Unused Imports/Functions**: Comment out imports and functions for disabled features to eliminate warnings
- **Conflicting Derives**: Remove duplicate derive attributes on structs
- **References to Disabled Enums**: Comment out or replace references to disabled enum variants

### Installation Failures
- **Permission Denied**: Suggest running as admin/sudo
- **PATH Issues**: Manually add to environment variables
- **Network Errors**: Retry downloads, check URLs
- **Binary Incompatible**: Rebuild for target architecture

### Iteration Process
1. **Identify Error**: Log exact error message
2. **Diagnose**: Check code, dependencies, environment
3. **Fix**: Apply targeted changes
4. **Test**: Re-run build/install
5. **Repeat**: If still fails, ask user for input or try alternatives
6. **Document**: Note fixes in docs/CHANGELOG.md

### Code Cleanup
- **Comment Out Disabled Features**: For features like multimodal, voice, comment out related code, imports, and enums to avoid errors and warnings
- **Remove Unused Code**: Delete or comment unused functions, structs, and variants after disabling features
- **Update Mod Files**: Ensure src/features/mod.rs only includes existing modules
- **Fix Line Endings**: Use consistent LF/CRLF as per platform when committing

## Testing and Quality Assurance
### Running Tests
```bash
cargo test
cargo clippy  # Linting
cargo fmt     # Formatting
```

### Manual Testing
- Pipe input: `echo "test" | echomind`
- Interactive: `echomind --interactive`
- Different providers: `echomind --provider openai --api-key KEY`
- Streaming: `echomind --stream`

## Deployment and Releases
### Pre-built Binaries
- `echomind-linux-x86_64` - Linux x86_64 binary (uncompressed, 3.6MB)
- `echomind-linux-x86_64.gz` - Linux x86_64 binary (gzip compressed, 1.7MB)
- These files are in repo root and can be curled directly
- Ensure cross-platform compatibility
- Test binaries before committing

### Quick Install URLs
```bash
# One-liner installer
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/curl-install.sh | bash

# Direct binary download
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/echomind-linux-x86_64.gz | gunzip | sudo tee /usr/bin/echomind > /dev/null && sudo chmod +x /usr/bin/echomind
```

### Version Management
- Update Cargo.toml version
- Update docs/CHANGELOG.md
- Tag releases: `git tag v0.x.x`

### CI/CD
- GitHub Actions in .github/workflows/
- Build on push/PR
- Release on tag

## User Interaction Guidelines
### Responding to Queries
- Be concise, direct, under 4 lines unless detail requested
- Use tools proactively for code analysis
- Explain non-trivial commands before running
- Avoid unnecessary output

### Handling User Requests
- **Code Changes**: Read files, plan edits, apply changes
- **Build/Install**: Run commands, handle errors gracefully
- **New Features**: Analyze requirements, implement incrementally
- **Bug Fixes**: Reproduce, fix, test
- **Documentation**: Update relevant .md files

### Asking for Clarification
- If request unclear, ask specific questions
- Provide examples of expected input
- Suggest alternatives if needed

## Advanced Features Context
- **Multimodal**: Images, PDFs via --image, --pdf
- **Voice**: Input/output with --voice-input, --voice-output
- **Batch Processing**: --batch for multiple queries
- **Benchmarking**: --benchmark for performance testing
- **Collaboration**: --share, --collaborate for sharing sessions
- **Security**: --encrypt, --audit-log for secure usage
- **TUI Chat**: Encrypted persistent chat history in TUI mode

## Emergency Procedures
- **Repo Corruption**: Re-clone, re-apply recent changes
- **Build Lock**: Delete target/, Cargo.lock, rebuild
- **Permission Issues**: Run as admin, check file ownership
- **Network Blocks**: Use local builds, offline mode
- **Critical Bugs**: Revert commits, isolate issues

This document ensures 100% context for any AI LLM handling the echomind project. Follow these instructions to maintain, enhance, and troubleshoot the codebase effectively.