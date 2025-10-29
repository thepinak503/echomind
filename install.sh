#!/bin/bash

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Echomind Installer ===${NC}"
echo ""

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

echo -e "${BLUE}Detected OS:${NC} $OS"
echo -e "${BLUE}Architecture:${NC} $ARCH"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust is not installed. Installing...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓ Rust installed${NC}"
fi

# Clone repository
TEMP_DIR="/tmp/echomind_install_$$"
echo -e "${BLUE}Cloning repository...${NC}"
git clone https://github.com/thepinak503/echomind.git "$TEMP_DIR"
cd "$TEMP_DIR"

# Build
echo -e "${BLUE}Building echomind...${NC}"
cargo build --release

# Install based on OS
case "$OS" in
    Linux*)
        echo -e "${BLUE}Installing for Linux...${NC}"

        # Detect package manager and distribution
        if command -v pacman &> /dev/null; then
            echo -e "${YELLOW}Arch Linux detected. You can use 'makepkg -si' instead.${NC}"
            echo -e "${YELLOW}Installing manually...${NC}"

        elif command -v apt &> /dev/null; then
            echo -e "${YELLOW}Debian/Ubuntu detected. Building .deb package...${NC}"
            # Install build dependencies if needed
            sudo apt-get update
            sudo apt-get install -y debhelper cargo rustc libssl-dev pkg-config

            # Build .deb package
            dpkg-buildpackage -us -uc -b

            # Install the package
            sudo dpkg -i ../echomind_0.3.0-1_*.deb || sudo apt-get install -f -y

            echo -e "${GREEN}✓ Installed via .deb package${NC}"
            cd /
            rm -rf "$TEMP_DIR"
            echo -e "${GREEN}✓ echomind installed successfully!${NC}"
            exit 0

        elif command -v dnf &> /dev/null; then
            echo -e "${YELLOW}Fedora/RHEL detected. Installing dependencies...${NC}"
            sudo dnf install -y cargo rust openssl-devel pkg-config
            echo -e "${YELLOW}Installing manually...${NC}"

        elif command -v yum &> /dev/null; then
            echo -e "${YELLOW}CentOS/RHEL detected. Installing dependencies...${NC}"
            sudo yum install -y cargo rust openssl-devel pkgconfig
            echo -e "${YELLOW}Installing manually...${NC}"

        elif command -v zypper &> /dev/null; then
            echo -e "${YELLOW}openSUSE detected. Installing dependencies...${NC}"
            sudo zypper install -y cargo rust libopenssl-devel pkg-config
            echo -e "${YELLOW}Installing manually...${NC}"

        elif command -v apk &> /dev/null; then
            echo -e "${YELLOW}Alpine Linux detected. Installing dependencies...${NC}"
            sudo apk add --no-cache cargo rust openssl-dev pkgconfig
            echo -e "${YELLOW}Installing manually...${NC}"

        else
            echo -e "${YELLOW}Unknown distribution. Installing manually...${NC}"
        fi

        # Manual installation
        sudo install -Dm755 target/release/echomind /usr/local/bin/echomind
        sudo install -Dm644 README.md /usr/local/share/doc/echomind/README.md
        sudo install -Dm644 CONTRIBUTING.md /usr/local/share/doc/echomind/CONTRIBUTING.md
        sudo install -Dm644 config.example.toml /usr/local/share/doc/echomind/config.example.toml
        sudo install -Dm644 echomind.1 /usr/local/share/man/man1/echomind.1
        sudo gzip -f /usr/local/share/man/man1/echomind.1

        echo -e "${GREEN}✓ Binary installed to /usr/local/bin/echomind${NC}"
        ;;

    Darwin*)
        echo -e "${BLUE}Installing for macOS...${NC}"

        # Check for Homebrew
        if command -v brew &> /dev/null; then
            echo -e "${YELLOW}You can also use Homebrew in the future!${NC}"
        fi

        # Install to /usr/local/bin
        sudo install -m 755 target/release/echomind /usr/local/bin/echomind
        sudo mkdir -p /usr/local/share/doc/echomind
        sudo install -m 644 README.md /usr/local/share/doc/echomind/README.md
        sudo install -m 644 CONTRIBUTING.md /usr/local/share/doc/echomind/CONTRIBUTING.md
        sudo install -m 644 config.example.toml /usr/local/share/doc/echomind/config.example.toml
        sudo mkdir -p /usr/local/share/man/man1
        sudo install -m 644 echomind.1 /usr/local/share/man/man1/echomind.1
        sudo gzip -f /usr/local/share/man/man1/echomind.1

        echo -e "${GREEN}✓ Binary installed to /usr/local/bin/echomind${NC}"
        ;;

    *)
        echo -e "${RED}Unsupported operating system: $OS${NC}"
        echo -e "${YELLOW}This script supports Linux and macOS only.${NC}"
        echo -e "${YELLOW}For Windows, use: irm -useb https://is.gd/echomindwin | iex${NC}"
        exit 1
        ;;
esac

# Clean up
cd /
rm -rf "$TEMP_DIR"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✓ echomind installed successfully!    ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Quick Start:${NC}"
echo -e "  ${YELLOW}1.${NC} Initialize config:  ${GREEN}echomind --init-config${NC}"
echo -e "  ${YELLOW}2.${NC} Try it out:        ${GREEN}echo 'Hello AI!' | echomind${NC}"
echo -e "  ${YELLOW}3.${NC} Interactive mode:  ${GREEN}echomind --interactive${NC}"
echo -e "  ${YELLOW}4.${NC} View help:         ${GREEN}echomind --help${NC}"
echo -e "  ${YELLOW}5.${NC} Man page:          ${GREEN}man echomind${NC}"
echo ""
echo -e "${BLUE}For more information:${NC}"
echo -e "  ${GREEN}https://github.com/thepinak503/echomind${NC}"
echo ""
