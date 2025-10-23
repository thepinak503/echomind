#!/bin/bash

set -e

echo "Installing echomind..."

# Check if on Arch Linux or derivative
if ! command -v pacman &> /dev/null; then
    echo "This script is designed for Arch Linux. Please use manual installation on other systems."
    exit 1
fi

# Clone the repository
git clone https://github.com/thepinak503/echomind.git /tmp/echomind_install
cd /tmp/echomind_install

# Build and install with makepkg
makepkg -si --noconfirm

# Clean up
cd /
rm -rf /tmp/echomind_install

echo "echomind installed successfully!"