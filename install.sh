#!/bin/bash

set -e

echo "Installing echomind..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi

# Clone the repository
git clone https://github.com/thepinak503/echomind.git /tmp/echomind_install
cd /tmp/echomind_install

# Build the project
cargo build --release

# Install the binary
sudo cp target/release/echomind /usr/local/bin/echomind

# Clean up
rm -rf /tmp/echomind_install

echo "echomind installed successfully! Run 'echomind --help' to get started."