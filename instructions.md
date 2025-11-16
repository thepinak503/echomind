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
├── install.sh              # Unix installer
├── echomind.exe            # Pre-built Windows executable
├── README.md               # Main documentation
├── INSTALL.md              # Installation guide
├── CONTRIBUTING.md         # Contribution guidelines
├── CHANGELOG.md            # Version history
├── RELEASE_NOTES.md        # Release notes
├── ENHANCED_FEATURES.md    # Feature documentation
├── config.example.toml     # Example configuration
├── PKGBUILD                # Arch Linux package
├── .SRCINFO                # Arch package info
├── echomind.1              # Man page
└── instructions.md         # This file
```

## Build Process
### Prerequisites
- Rust 1.70+ (`cargo`, `rustc`)
- For Windows: MSVC compiler (Visual Studio Build Tools)
- For Linux/macOS: Standard development tools

### Build Commands
```bash
# Clone repository
git clone https://github.com/thepinak503/echomind.git
cd echomind

# Build in release mode
cargo build --release

# Binary location: target/release/echomind (or .exe on Windows)
```

### Cross-Platform Builds
- Linux: `cargo build --release`
- macOS: `cargo build --release`
- Windows: Requires MSVC; fallback to pre-built exe if build fails

## Installation Process
### Automated Installers
- **Windows**: `install.ps1` - Installs Rust if needed, builds or falls back to repo exe, copies to user bin and optionally System32
- **Unix**: `install.sh` - Downloads and installs pre-built binaries

### Manual Installation
1. Build the project
2. Copy binary to PATH (e.g., `~/.local/bin/echomind`)
3. Initialize config: `echomind --init-config`

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
4. **Document**: Update README.md, ENHANCED_FEATURES.md
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
- **Config**: Update config.example.toml, add parsing

## Error Handling and Iteration
### Build Failures
- **MSVC Missing**: Warn, fallback to pre-built exe
- **Dependency Issues**: Update Cargo.lock, check versions
- **Compilation Errors**: Fix syntax, add missing imports
- **Linker Errors**: Check system libraries, update build scripts

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
6. **Document**: Note fixes in CHANGELOG.md

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
- Update `echomind.exe` in repo root with latest build
- Ensure cross-platform compatibility
- Test binaries before committing

### Version Management
- Update Cargo.toml version
- Update CHANGELOG.md
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

## Emergency Procedures
- **Repo Corruption**: Re-clone, re-apply recent changes
- **Build Lock**: Delete target/, Cargo.lock, rebuild
- **Permission Issues**: Run as admin, check file ownership
- **Network Blocks**: Use local builds, offline mode
- **Critical Bugs**: Revert commits, isolate issues

This document ensures 100% context for any AI LLM handling the echomind project. Follow these instructions to maintain, enhance, and troubleshoot the codebase effectively.