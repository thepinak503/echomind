# Build Guide for EchoMind

This guide explains how to build EchoMind optimally for your platform.

## Prerequisites

- **Rust**: Install from https://rustup.rs/
- **Git**: For cloning the repository
- **Platform-specific requirements**:
  - **Linux**: `build-essential`, `libssl-dev`, `pkg-config`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Visual Studio Build Tools or Visual Studio Community

## Quick Build

```bash
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release
```

The binary will be at `target/release/echomind` (or `echomind.exe` on Windows).

## Feature-Based Builds

### Minimal Build (No Optional Features)
```bash
cargo build --release --no-default-features
```

### Full Features Build
```bash
cargo build --release --all-features
```

### Selective Features
```bash
# With voice support (audio input/output)
cargo build --release --features voice

# With multimodal support (images, PDFs, documents)
cargo build --release --features pdf,images

# With voice and multimodal
cargo build --release --features voice,pdf,images
```

## Platform-Specific Builds

### Linux (All Distributions)

```bash
# Ubuntu/Debian
sudo apt-get install -y build-essential libssl-dev pkg-config

# Fedora/RHEL
sudo dnf install -y gcc libssl-devel pkg-config

# Arch
sudo pacman -S base-devel openssl pkg-config

# Build
cargo build --release --all-features
```

**Notes for Linux:**
- **Wayland Support**: Built-in via crossterm
- **X11 Support**: Automatic fallback clipboard support
- **Voice Support**: Requires PulseAudio or ALSA

### macOS (Intel & Apple Silicon)

```bash
# Install Xcode Command Line Tools (if not already installed)
xcode-select --install

# Build for current architecture (automatic)
cargo build --release --all-features

# Build for Apple Silicon specifically
cargo build --release --all-features --target aarch64-apple-darwin

# Build for Intel specifically
cargo build --release --all-features --target x86_64-apple-darwin

# Build universal binary (Intel + Apple Silicon)
cargo build --release --all-features --target aarch64-apple-darwin
cargo build --release --all-features --target x86_64-apple-darwin
# Then use lipo to create universal binary:
# lipo -create target/aarch64-apple-darwin/release/echomind target/x86_64-apple-darwin/release/echomind -output echomind-universal
```

**Notes for macOS:**
- Automatic audio backend detection (CoreAudio)
- Native Cocoa clipboard support
- Optimized for both Intel and Apple Silicon

### Windows (x86_64)

```bash
# Using PowerShell (recommended)
cargo build --release --all-features

# For WSL (Windows Subsystem for Linux)
# Follow the Linux build instructions above
```

**Notes for Windows:**
- **Console Support**: Full Windows Console API support via crossterm
- **PowerShell**: Works with Windows Terminal and legacy cmd.exe
- **Audio**: Uses Windows Audio Session API (WASAPI)
- **Clipboard**: Native Windows clipboard via Win32 API

## Optimization Options

### Aggressive Release Optimization
The `Cargo.toml` includes optimized release settings:
- Full LTO (Link-Time Optimization)
- Single codegen unit
- Symbol stripping

For even smaller binaries:
```bash
cargo build --release -Z build-std=core,std --target x86_64-unknown-linux-gnu
```

### Faster Development Builds
```bash
cargo build
```

Produces an unoptimized binary in `target/debug/echomind`.

## Installation After Build

### Linux/macOS
```bash
sudo cp target/release/echomind /usr/local/bin/
# or
cargo install --path .
```

### Windows
Copy `target/release/echomind.exe` to a directory in your `PATH`, or use:
```powershell
cargo install --path .
```

## Troubleshooting

### "error: linker cc not found"
**Linux**: Install build tools
```bash
# Ubuntu/Debian
sudo apt-get install build-essential
```

### "error: failed to run custom build command"
**All platforms**: Update Rust
```bash
rustup update
```

### OpenSSL/SSL Errors
**macOS**:
```bash
# Using Homebrew
brew install openssl
export LDFLAGS="-L$(brew --prefix)/opt/openssl@3/lib"
export CPPFLAGS="-I$(brew --prefix)/opt/openssl@3/include"
```

**Linux**:
```bash
sudo apt-get install libssl-dev pkg-config
```

### Audio Feature Build Issues

If `cpal` or `rodio` fail to build:

```bash
# Windows/macOS: Usually works out of the box
cargo build --release --features voice

# Linux: Install audio libraries
sudo apt-get install libasound2-dev  # ALSA headers
# or
sudo apt-get install libpulse-dev    # PulseAudio headers
```

## Cross-Compilation

Build for a different platform:

```bash
# Add target
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

## Performance Benchmarking

To test performance:

```bash
# Time a simple query
time echo "Hello" | ./target/release/echomind

# Benchmark with hyperfine (requires: cargo install hyperfine)
hyperfine './target/release/echomind --model gpt-3.5-turbo <<< "test"'
```

## Docker Build

If you prefer containerization:

```dockerfile
FROM rust:latest

WORKDIR /build
COPY . .
RUN cargo build --release --all-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=0 /build/target/release/echomind /usr/local/bin/

ENTRYPOINT ["echomind"]
```

Build and run:
```bash
docker build -t echomind .
docker run -e ECHOMIND_API_KEY=your_key echomind --version
```

## Next Steps

After building, see [CONTRIBUTING.md](CONTRIBUTING.md) for development setup.
