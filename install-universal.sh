#!/bin/bash
# Universal installer for EchoMind
# Supports Linux, macOS, and WSL

set -e

REPO="https://github.com/thepinak503/echomind.git"
VERSION="latest"
FEATURES="--all-features"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

detect_platform() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            DISTRO="$ID"
        else
            DISTRO="linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        DISTRO="darwin"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
        OS="windows"
        DISTRO="windows"
    else
        OS="unknown"
        DISTRO="unknown"
    fi
}

check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_warning "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
        print_success "Rust installed"
    else
        print_success "Rust found: $(rustc --version)"
    fi
}

install_linux_deps() {
    print_info "Installing dependencies for $DISTRO..."

    case "$DISTRO" in
        ubuntu|debian)
            sudo apt-get update
            sudo apt-get install -y build-essential libssl-dev pkg-config
            # Optional: audio support
            if [[ "$FEATURES" == *"voice"* ]]; then
                sudo apt-get install -y libasound2-dev libpulse-dev
            fi
            ;;
        fedora|rhel|centos)
            sudo dnf install -y gcc libssl-devel pkg-config
            if [[ "$FEATURES" == *"voice"* ]]; then
                sudo dnf install -y alsa-lib-devel pulseaudio-libs-devel
            fi
            ;;
        arch|manjaro)
            sudo pacman -S --noconfirm base-devel openssl pkg-config
            if [[ "$FEATURES" == *"voice"* ]]; then
                sudo pacman -S --noconfirm alsa-lib pulseaudio
            fi
            ;;
        *)
            print_warning "Unknown distro: $DISTRO. Skipping automatic dependency installation."
            print_info "Install: build-essential, libssl-dev, pkg-config"
            ;;
    esac
    print_success "Dependencies installed"
}

install_macos_deps() {
    print_info "Checking macOS dependencies..."

    if ! command -v xcode-select &> /dev/null; then
        print_info "Installing Xcode Command Line Tools..."
        xcode-select --install
        print_success "Xcode Command Line Tools installed"
    else
        print_success "Xcode Command Line Tools found"
    fi

    # Check for Homebrew
    if ! command -v brew &> /dev/null; then
        print_warning "Homebrew not found. Some optional features may not work."
    else
        print_info "Installing additional dependencies via Homebrew..."
        brew install openssl
        print_success "Dependencies installed"
    fi
}

build_echomind() {
    print_info "Cloning EchoMind repository..."

    if [ -d "echomind" ]; then
        print_warning "echomind directory exists. Updating..."
        cd echomind
        git pull
    else
        git clone "$REPO"
        cd echomind
    fi

    print_success "Repository ready"

    print_info "Building EchoMind with features: $FEATURES..."
    print_info "This may take a few minutes..."

    if cargo build --release $FEATURES 2>&1 | tee build.log; then
        print_success "Build successful"
        return 0
    else
        print_error "Build failed. See build.log for details."
        return 1
    fi
}

install_binary() {
    print_info "Installing EchoMind binary..."

    case "$OS" in
        linux|macos)
            BINARY="target/release/echomind"
            INSTALL_DIR="/usr/local/bin"
            ;;
        windows)
            BINARY="target/release/echomind.exe"
            INSTALL_DIR="$APPDATA/Local/Programs/echomind"
            ;;
    esac

    if [ ! -f "$BINARY" ]; then
        print_error "Binary not found: $BINARY"
        return 1
    fi

    if [[ "$OS" == "linux" ]] || [[ "$OS" == "macos" ]]; then
        sudo cp "$BINARY" "$INSTALL_DIR/"
        sudo chmod +x "$INSTALL_DIR/echomind"
        print_success "Installed to $INSTALL_DIR/echomind"
    else
        mkdir -p "$INSTALL_DIR"
        cp "$BINARY" "$INSTALL_DIR/"
        print_success "Installed to $INSTALL_DIR/echomind.exe"
    fi
}

verify_installation() {
    print_info "Verifying installation..."

    if command -v echomind &> /dev/null; then
        VERSION_OUTPUT=$(echomind --version 2>/dev/null || echo "unknown")
        print_success "EchoMind installed successfully: $VERSION_OUTPUT"
        return 0
    else
        print_warning "echomind not found in PATH"
        print_info "You can run it from: $(pwd)/$BINARY"
        return 1
    fi
}

show_usage() {
    print_info "Quick start guide:"
    echo ""
    echo "  1. Configure API:"
    echo "     echomind --init-config"
    echo ""
    echo "  2. Test it:"
    echo "     echo 'Hello, AI!' | echomind"
    echo ""
    echo "  3. Interactive mode:"
    echo "     echomind --interactive"
    echo ""
    echo "  4. TUI mode:"
    echo "     echomind --tui"
    echo ""
    echo "For more help:"
    echo "     echomind --help"
}

main() {
    echo "╔════════════════════════════════════════╗"
    echo "║   EchoMind Universal Installer        ║"
    echo "║   Cross-Platform AI CLI Tool          ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    detect_platform
    print_info "Detected platform: $OS ($DISTRO)"

    # Check for Rust
    check_rust

    # Install platform-specific dependencies
    case "$OS" in
        linux)
            install_linux_deps
            ;;
        macos)
            install_macos_deps
            ;;
        windows)
            print_warning "Windows: Please ensure Visual Studio Build Tools are installed"
            ;;
    esac

    # Build from source
    if ! build_echomind; then
        print_error "Installation failed during build"
        exit 1
    fi

    # Install binary
    if ! install_binary; then
        print_warning "Could not install to system directory"
        print_info "You can run from: $(pwd)/$BINARY"
    fi

    # Verify
    verify_installation

    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║   Installation Complete! 🎉            ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    show_usage
}

# Run main
main "$@"
