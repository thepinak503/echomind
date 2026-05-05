#!/bin/bash
set -e

echo "🚀 Installing EchoMind v0.3.5..."

REPO="thepinak503/echomind"
BINARY="echomind"

TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

cd "$TMP_DIR"

echo "📦 Downloading binary..."
curl -fsSL "https://raw.githubusercontent.com/${REPO}/master/echomind-linux-x86_64.gz" -o echomind.gz

echo "📂 Extracting..."
gunzip echomind.gz
chmod +x echomind

echo "🔧 Installing to /usr/bin..."
if [ -w /usr/bin ]; then
    mv echomind /usr/bin/echomind
else
    sudo mv echomind /usr/bin/echomind
fi

echo "✅ Installation complete!"
echo ""
echo "Run 'echomind --help' to get started."
echo "Run 'echomind --tui' for interactive TUI mode."
