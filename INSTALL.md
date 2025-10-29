# Installation Guide

Complete installation instructions for echomind across all platforms.

## 📋 Table of Contents

- [Linux](#-linux)
  - [Arch Linux](#arch-linux)
  - [Debian/Ubuntu](#debianubuntu)
  - [From Source](#linux-from-source)
- [macOS](#-macos)
- [Windows](#-windows)
- [Docker](#-docker)
- [Post-Installation](#-post-installation)

---

## 🐧 Linux

### Arch Linux

#### Method 1: Using PKGBUILD (Recommended)

```bash
# Clone the repository
git clone https://github.com/thepinak503/echomind.git
cd echomind

# Build and install
makepkg -si
```

This will:
- Build the package with cargo
- Create `echomind-0.3.0-1-x86_64.pkg.tar.zst`
- Install it via pacman
- Install to `/usr/bin/echomind`

#### Method 2: From AUR (Coming Soon)

```bash
# Using yay
yay -S echomind

# Or using paru
paru -S echomind
```

### Debian/Ubuntu

#### Method 1: Build .deb Package (Recommended)

```bash
# Install build dependencies
sudo apt update
sudo apt install -y debhelper cargo rustc libssl-dev pkg-config git

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
dpkg-buildpackage -us -uc -b

# Install the package
sudo dpkg -i ../echomind_0.3.0-1_amd64.deb

# Fix any missing dependencies
sudo apt-get install -f
```

Files will be installed to:
- Binary: `/usr/bin/echomind`
- Docs: `/usr/share/doc/echomind/`
- Man page: `/usr/share/man/man1/echomind.1.gz`

#### Method 2: Pre-built .deb (Coming Soon)

```bash
# Download from releases
wget https://github.com/thepinak503/echomind/releases/download/v0.3.0/echomind_0.3.0-1_amd64.deb

# Install
sudo dpkg -i echomind_0.3.0-1_amd64.deb
sudo apt-get install -f
```

### Fedora / RHEL 9+

#### Method 1: From Source

```bash
# Install dependencies
sudo dnf install -y cargo rust openssl-devel pkg-config git

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Install
sudo install -Dm755 target/release/echomind /usr/local/bin/echomind
sudo install -Dm644 README.md /usr/local/share/doc/echomind/README.md
sudo install -Dm644 echomind.1 /usr/local/share/man/man1/echomind.1
sudo gzip -f /usr/local/share/man/man1/echomind.1
```

#### Method 2: RPM Package (Future)

RPM packages for Fedora and RHEL are planned for future releases.

### CentOS / RHEL 8

```bash
# Install dependencies (may need EPEL repository)
sudo yum install -y cargo rust openssl-devel pkgconfig git

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Install
sudo install -Dm755 target/release/echomind /usr/local/bin/echomind
sudo install -Dm644 README.md /usr/local/share/doc/echomind/README.md
sudo install -Dm644 echomind.1 /usr/local/share/man/man1/echomind.1
sudo gzip -f /usr/local/share/man/man1/echomind.1
```

**Note:** On CentOS/RHEL 8, you may need to enable EPEL and PowerTools/CodeReady Builder repositories for Rust:
```bash
sudo yum install -y epel-release
sudo yum config-manager --set-enabled powertools  # CentOS 8
# OR
sudo yum config-manager --set-enabled crb  # RHEL 8
```

### openSUSE

#### For openSUSE Leap 15.3+ and Tumbleweed

```bash
# Install dependencies
sudo zypper install -y cargo rust libopenssl-devel pkg-config git

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Install
sudo install -m 755 target/release/echomind /usr/local/bin/echomind
sudo mkdir -p /usr/local/share/doc/echomind
sudo install -m 644 README.md /usr/local/share/doc/echomind/README.md
sudo mkdir -p /usr/local/share/man/man1
sudo install -m 644 echomind.1 /usr/local/share/man/man1/echomind.1
sudo gzip -f /usr/local/share/man/man1/echomind.1
```

### Alpine Linux

#### Lightweight Installation

```bash
# Install dependencies
sudo apk add --no-cache cargo rust openssl-dev pkgconfig git musl-dev

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Install
sudo install -Dm755 target/release/echomind /usr/local/bin/echomind
sudo install -Dm644 README.md /usr/local/share/doc/echomind/README.md
sudo install -Dm644 echomind.1 /usr/local/share/man/man1/echomind.1
sudo gzip -f /usr/local/share/man/man1/echomind.1
```

**Note:** Alpine uses musl libc, which is fully compatible with echomind.

### Linux: Universal From Source

#### Automatic Installation Script

```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

This script will:
- Detect your OS and package manager
- Install Rust if needed
- Build echomind
- Install to the appropriate location

#### Manual Installation

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Install manually
sudo install -Dm755 target/release/echomind /usr/local/bin/echomind
sudo install -Dm644 README.md /usr/local/share/doc/echomind/README.md
sudo install -Dm644 echomind.1 /usr/local/share/man/man1/echomind.1
sudo gzip -f /usr/local/share/man/man1/echomind.1
```

---

## 🍎 macOS

### Method 1: Homebrew (Coming Soon)

```bash
brew tap thepinak503/echomind
brew install echomind
```

### Method 2: Pre-built Binary (Coming Soon)

```bash
# Download for your architecture
# For Intel Macs:
wget https://github.com/thepinak503/echomind/releases/download/v0.3.0/echomind-macos-amd64
chmod +x echomind-macos-amd64
sudo mv echomind-macos-amd64 /usr/local/bin/echomind

# For Apple Silicon (M1/M2/M3):
wget https://github.com/thepinak503/echomind/releases/download/v0.3.0/echomind-macos-arm64
chmod +x echomind-macos-arm64
sudo mv echomind-macos-arm64 /usr/local/bin/echomind
```

### Method 3: From Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Install
sudo install -m 755 target/release/echomind /usr/local/bin/echomind
sudo mkdir -p /usr/local/share/doc/echomind
sudo install -m 644 README.md /usr/local/share/doc/echomind/README.md
sudo mkdir -p /usr/local/share/man/man1
sudo install -m 644 echomind.1 /usr/local/share/man/man1/echomind.1
sudo gzip -f /usr/local/share/man/man1/echomind.1
```

### Method 4: Automatic Installation Script

```bash
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

---

## 🪟 Windows

### Method 1: Automatic Installation (PowerShell)

**Recommended for most users:**

```powershell
irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex
```

This will:
- Install Rust if needed
- Clone and build echomind
- Install to `$HOME\.local\bin\`
- Add to your PATH automatically
- Set up documentation

**Note:** You may need to restart your terminal after installation.

### Method 2: Pre-built Binary (Recommended - Quick Install)

**Run as Administrator for system-wide installation:**

```powershell
# Download the executable using curl
curl -L https://github.com/thepinak503/echomind/raw/master/echomind-windows-x86_64.exe -o echomind.exe

# Option A: Install to System32 (requires admin)
Move-Item echomind.exe C:\Windows\System32\echomind.exe

# Option B: Install to user directory (no admin required)
$installDir = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Path $installDir -Force
Move-Item echomind.exe "$installDir\echomind.exe"

# Add to PATH (user environment variable)
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$path;$installDir", "User")
    Write-Host "Added $installDir to PATH. Please restart your terminal."
}

# Verify installation
echomind --version
```

**Alternative using Invoke-WebRequest:**

```powershell
# Download
Invoke-WebRequest -Uri "https://github.com/thepinak503/echomind/raw/master/echomind-windows-x86_64.exe" -OutFile "echomind.exe"

# Install to user directory
$installDir = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Path $installDir -Force
Move-Item echomind.exe "$installDir\echomind.exe" -Force

# Update PATH
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$path;$installDir", "User")
}
```

### Method 3: Using Scoop (Coming Soon)

```powershell
scoop bucket add echomind https://github.com/thepinak503/echomind
scoop install echomind
```

### Method 4: Using Cargo (Rust Package Manager)

```powershell
# Install Rust from https://rustup.rs/
# Then install echomind:
cargo install --git https://github.com/thepinak503/echomind
```

### Method 5: WSL (Windows Subsystem for Linux)

If you have WSL installed, follow the Linux instructions:

```bash
# In WSL terminal
curl -fsSL https://raw.githubusercontent.com/thepinak503/echomind/master/install.sh | bash
```

### Method 6: From Source

```powershell
# Install Rust from https://rustup.rs/

# Clone and build
git clone https://github.com/thepinak503/echomind.git
cd echomind
cargo build --release

# Binary will be at: target\release\echomind.exe
# Copy it to a directory in your PATH
```

---

## 🐳 Docker

### Using Docker

```bash
# Build the image
docker build -t echomind https://github.com/thepinak503/echomind.git

# Run echomind
echo "Hello AI!" | docker run -i --rm echomind

# Interactive mode
docker run -it --rm echomind --interactive

# With config file
docker run -i --rm -v ~/.config/echomind:/root/.config/echomind echomind
```

### Dockerfile Example

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/echomind /usr/local/bin/echomind
ENTRYPOINT ["echomind"]
```

---

## ✅ Post-Installation

### Verify Installation

```bash
# Check version
echomind --version

# View help
echomind --help

# Read man page (Linux/macOS)
man echomind
```

### Initial Configuration

```bash
# Create default config
echomind --init-config

# View config location
echomind --show-config

# Edit config (example for Linux)
nano ~/.config/echomind/config.toml
```

### Example Config

```toml
[api]
provider = "chatanywhere"
api_key = "your-api-key-here"
model = "gpt-3.5-turbo"
timeout = 30

[defaults]
temperature = 0.7
coder_mode = false
stream = false
```

### Set API Key

Choose one method:

**1. Config File:**
```bash
echomind --init-config
# Then edit ~/.config/echomind/config.toml
```

**2. Environment Variable:**
```bash
export ECHOMIND_API_KEY="your-api-key"
```

**3. Command Line:**
```bash
echo "Hello" | echomind --api-key "your-api-key" --provider openai
```

### First Test

```bash
# Simple test
echo "Say hello in 3 languages" | echomind

# Interactive mode
echomind --interactive

# Coder mode
echo "write a Python hello world" | echomind --coder

# With streaming
echo "Tell me a short story" | echomind --stream
```

---

## 🔧 Build Requirements

### All Platforms

- **Rust**: 1.70 or later
- **Cargo**: Rust's package manager
- **Git**: For cloning the repository

### Linux

- **Debian/Ubuntu**: `libssl-dev`, `pkg-config`
- **Arch/Manjaro**: `openssl`, `pkg-config`
- **Fedora/RHEL**: `openssl-devel`, `pkg-config`
- **CentOS/Rocky/Alma**: `openssl-devel`, `pkgconfig`
- **openSUSE**: `libopenssl-devel`, `pkg-config`
- **Alpine**: `openssl-dev`, `pkgconfig`, `musl-dev`

### macOS

- **Xcode Command Line Tools**: `xcode-select --install`
- **Homebrew** (optional): For easier dependency management

### Windows

- **Visual Studio Build Tools** or **MinGW-w64**
- **OpenSSL**: Can be installed via vcpkg

---

## 📦 Package Managers

| Platform | Package Manager | Status |
|----------|----------------|--------|
| Arch Linux / Manjaro | pacman (via PKGBUILD) | ✅ Available |
| Arch Linux | AUR | 🔜 Coming Soon |
| Debian / Ubuntu | dpkg/apt (.deb) | ✅ Available |
| Debian / Ubuntu | PPA | 🔜 Coming Soon |
| Fedora / RHEL | dnf/yum (manual) | ✅ Available |
| Fedora | rpm | 🔜 Coming Soon |
| CentOS / Rocky / Alma | yum/dnf (manual) | ✅ Available |
| openSUSE | zypper (manual) | ✅ Available |
| Alpine Linux | apk (manual) | ✅ Available |
| macOS | Homebrew | 🔜 Coming Soon |
| Windows | Scoop | 🔜 Coming Soon |
| Windows | Chocolatey | 🔜 Coming Soon |
| Any | Cargo | ✅ Available |
| Any Linux | install.sh script | ✅ Available |

---

## 🆘 Troubleshooting

### Linux: Permission Denied

```bash
sudo chmod +x /usr/bin/echomind
# or
sudo chmod +x /usr/local/bin/echomind
```

### macOS: "echomind" cannot be opened

```bash
xattr -d com.apple.quarantine /usr/local/bin/echomind
```

### Windows: Not Recognized

Make sure the directory containing echomind.exe is in your PATH:

```powershell
$env:PATH += ";$HOME\.local\bin"
# Make permanent:
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, [EnvironmentVariableTarget]::User)
```

### Build Errors

```bash
# Update Rust
rustup update

# Clean build
cargo clean
cargo build --release
```

---

## 🔗 Links

- **Repository**: https://github.com/thepinak503/echomind
- **Issues**: https://github.com/thepinak503/echomind/issues
- **Releases**: https://github.com/thepinak503/echomind/releases
- **Documentation**: https://github.com/thepinak503/echomind/blob/master/README.md

---

## 📝 License

MIT License - see [LICENSE](LICENSE) file for details.
