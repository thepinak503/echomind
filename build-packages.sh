#!/usr/bin/env bash
set -euo pipefail

# ── Configuration ──────────────────────────────────────────────
NAME="echomind"
VERSION="0.3.5"
MAINTAINER="Pinak Dhabu <thepinak503@duck.com>"
DESC="AI-powered CLI tool with multiple provider support, streaming, and interactive mode"
HOME_URL="https://github.com/thepinak503/echomind"

ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  DEB_ARCH="amd64"; RUST_TARGET="x86_64-unknown-linux-gnu" ;;
  aarch64) DEB_ARCH="arm64";  RUST_TARGET="aarch64-unknown-linux-gnu" ;;
  armv7l)  DEB_ARCH="armhf"; RUST_TARGET="armv7-unknown-linux-gnueabihf" ;;
  *)       DEB_ARCH="$ARCH";  RUST_TARGET="$ARCH-unknown-linux-gnu" ;;
esac

ROOT="$(cd "$(dirname "$0")" && pwd)"
BUILD="$ROOT/build"
PKG="$BUILD/packaging"

# ── Clean ──────────────────────────────────────────────────────
clean() {
  echo "==> Cleaning..."
  rm -rf "$BUILD"
  cargo clean 2>/dev/null || true
}

# ── Build Release Binary ───────────────────────────────────────
build_release() {
  echo "==> Building release binary..."
  cd "$ROOT"
  cargo build --release
  mkdir -p "$PKG"
}

# ── Arch Linux Package (.pkg.tar.zst) ──────────────────────────
build_arch() {
  echo "==> Building Arch Linux package..."
  local d="$PKG/arch/$NAME"
  mkdir -p "$d/usr/bin"
  mkdir -p "$d/usr/share/doc/$NAME"
  mkdir -p "$d/usr/share/man/man1"
  mkdir -p "$d/usr/share/licenses/$NAME"
  mkdir -p "$d/usr/share/bash-completion/completions"
  mkdir -p "$d/usr/share/zsh/vendor-completions"
  mkdir -p "$d/usr/share/fish/vendor_completions.d"

  install -Dm755 "$ROOT/target/release/$NAME" "$d/usr/bin/$NAME"
  install -Dm644 "$ROOT/README.md" "$d/usr/share/doc/$NAME/README.md"
  install -Dm644 "$ROOT/CONTRIBUTING.md" "$d/usr/share/doc/$NAME/CONTRIBUTING.md"
  install -Dm644 "$ROOT/docs/config.example.toml" "$d/usr/share/doc/$NAME/config.example.toml"
  install -Dm644 "$ROOT/docs/CHANGELOG.md" "$d/usr/share/doc/$NAME/CHANGELOG.md"
  install -Dm644 "$ROOT/instructions.md" "$d/usr/share/doc/$NAME/instructions.md"
  install -Dm644 "$ROOT/$NAME.1" "$d/usr/share/man/man1/$NAME.1"
  gzip -nf "$d/usr/share/man/man1/$NAME.1"
  install -Dm644 "$ROOT/docs/LICENSE" "$d/usr/share/licenses/$NAME/LICENSE"
  [ -f "$ROOT/docs/completions/$NAME.bash" ] && install -Dm644 "$ROOT/docs/completions/$NAME.bash" "$d/usr/share/bash-completion/completions/$NAME"
  [ -f "$ROOT/docs/completions/_$NAME" ] && install -Dm644 "$ROOT/docs/completions/_$NAME" "$d/usr/share/zsh/vendor-completions/_$NAME"
  [ -f "$ROOT/docs/completions/$NAME.fish" ] && install -Dm644 "$ROOT/docs/completions/$NAME.fish" "$d/usr/share/fish/vendor_completions.d/$NAME.fish"

  local pkgfile="$ROOT/$NAME-$VERSION-1-$ARCH.pkg.tar.zst"
  cd "$d" && tar --zstd -cf "$pkgfile" .
  echo "  ✅ $pkgfile"
}

# ── Debian Package (.deb) ──────────────────────────────────────
build_deb() {
  echo "==> Building Debian package..."
  local d="$PKG/deb/${NAME}_${VERSION}-1_${DEB_ARCH}"
  mkdir -p "$d/DEBIAN"
  mkdir -p "$d/usr/bin"
  mkdir -p "$d/usr/share/doc/$NAME"
  mkdir -p "$d/usr/share/man/man1"
  mkdir -p "$d/usr/share/licenses/$NAME"
  mkdir -p "$d/usr/share/bash-completion/completions"
  mkdir -p "$d/usr/share/zsh/vendor-completions"
  mkdir -p "$d/usr/share/fish/vendor_completions.d"

  install -Dm755 "$ROOT/target/release/$NAME" "$d/usr/bin/$NAME"
  install -Dm644 "$ROOT/README.md" "$d/usr/share/doc/$NAME/README.md"
  install -Dm644 "$ROOT/CONTRIBUTING.md" "$d/usr/share/doc/$NAME/CONTRIBUTING.md"
  install -Dm644 "$ROOT/docs/config.example.toml" "$d/usr/share/doc/$NAME/config.example.toml"
  install -Dm644 "$ROOT/docs/CHANGELOG.md" "$d/usr/share/doc/$NAME/CHANGELOG.md"
  install -Dm644 "$ROOT/docs/RELEASE_NOTES.md" "$d/usr/share/doc/$NAME/RELEASE_NOTES.md"
  install -Dm644 "$ROOT/docs/LICENSE" "$d/usr/share/licenses/$NAME/LICENSE"
  install -Dm644 "$ROOT/$NAME.1" "$d/usr/share/man/man1/$NAME.1"
  gzip -nf "$d/usr/share/man/man1/$NAME.1" 2>/dev/null || true
  [ -f "$ROOT/docs/completions/$NAME.bash" ] && install -Dm644 "$ROOT/docs/completions/$NAME.bash" "$d/usr/share/bash-completion/completions/$NAME"
  [ -f "$ROOT/docs/completions/_$NAME" ] && install -Dm644 "$ROOT/docs/completions/_$NAME" "$d/usr/share/zsh/vendor-completions/_$NAME"
  [ -f "$ROOT/docs/completions/$NAME.fish" ] && install -Dm644 "$ROOT/docs/completions/$NAME.fish" "$d/usr/share/fish/vendor_completions.d/$NAME.fish"

  local deb_size
  deb_size=$(du -sk "$d" | cut -f1)

  cat > "$d/DEBIAN/control" <<EOF
Package: $NAME
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: $MAINTAINER
Installed-Size: $deb_size
Depends: libc6 (>= 2.34), libssl3 (>= 3.0.2)
Recommends: xclip, xsel, wl-clipboard
Homepage: $HOME_URL
Description: $DESC
 A command-line interface for AI chat APIs including OpenAI, Claude,
 Gemini, Ollama, Grok, Mistral, Cohere, and ChatAnywhere. Features
 streaming responses, interactive mode, TUI interface, and advanced
 options for temperature, tokens, and model selection.
EOF

  local debfile="$ROOT/${NAME}_${VERSION}-1_${DEB_ARCH}.deb"
  dpkg-deb --build "$d" "$ROOT" 2>/dev/null
  echo "  ✅ $debfile"
}

# ── Install from local package ────────────────────────────────
install_local() {
  if command -v pacman &>/dev/null; then
    local pkgfile="$ROOT/$NAME-$VERSION-1-$ARCH.pkg.tar.zst"
    if [ -f "$pkgfile" ]; then
      echo "==> Installing via pacman -U..."
      sudo pacman -U --noconfirm "$pkgfile"
    else
      echo "==> Building Arch package first..."
      build_arch
      sudo pacman -U --noconfirm "$ROOT/$NAME-$VERSION-1-$ARCH.pkg.tar.zst"
    fi
  elif command -v dpkg &>/dev/null; then
    local debfile="$ROOT/${NAME}_${VERSION}-1_${DEB_ARCH}.deb"
    if [ -f "$debfile" ]; then
      echo "==> Installing via dpkg..."
      sudo dpkg -i "$debfile"
    else
      echo "==> Building Debian package first..."
      build_deb
      sudo dpkg -i "$ROOT/${NAME}_${VERSION}-1_${DEB_ARCH}.deb"
    fi
  else
    echo "==> No package manager found. Just building binary..."
    build_release
    sudo cp "$ROOT/target/release/$NAME" /usr/bin/
  fi
}

# ── Help ───────────────────────────────────────────────────────
usage() {
  echo "Usage: $0 {clean|build|arch|deb|install|all}"
  echo ""
  echo "  clean    Remove build artifacts"
  echo "  build    Build release binary"
  echo "  arch     Build Arch Linux package (.pkg.tar.zst)"
  echo "  deb      Build Debian package (.deb)"
  echo "  install  Build & install for your system"
  echo "  all      Build everything (default)"
  exit 1
}

case "${1:-all}" in
  clean)    clean ;;
  build)    clean; build_release ;;
  arch)     clean; build_release; build_arch ;;
  deb)      clean; build_release; build_deb ;;
  install)  clean; build_release; install_local ;;
  all)      clean; build_release; build_arch; build_deb ;;
  *)        usage ;;
esac
