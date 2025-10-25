#!/bin/bash

set -e

echo "Installing echomind..."

# Clone the repository
git clone https://github.com/thepinak503/echomind.git /tmp/echomind_install
cd /tmp/echomind_install/src/echomind-0.1.0

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Build the project
cargo build --release

# Install the binary
sudo cp target/release/echomind /usr/local/bin/

# Install man page
sudo mkdir -p /usr/local/share/man/man1
sudo cp echomind.1 /usr/local/share/man/man1/
sudo gzip /usr/local/share/man/man1/echomind.1

# Install documentation
sudo mkdir -p /usr/local/share/doc/echomind
sudo cp README.md /usr/local/share/doc/echomind/

# Clean up
cd /
rm -rf /tmp/echomind_install

echo "echomind installed successfully!"