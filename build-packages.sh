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
  x86_64)  DEB_ARCH="amd64"; PKG_ARCH="x86_64" ;;
  aarch64) DEB_ARCH="arm64";  PKG_ARCH="aarch64" ;;
  armv7l)  DEB_ARCH="armhf"; PKG_ARCH="armv7l" ;;
  *)       DEB_ARCH="$ARCH";  PKG_ARCH="$ARCH" ;;
esac

ROOT="$(cd "$(dirname "$0")" && pwd)"

# ── Colors ─────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
ok()  { echo -e "${GREEN}  ✅ $1${NC}"; }
warn(){ echo -e "${YELLOW}  ⚠️  $1${NC}"; }
fail(){ echo -e "${RED}  ❌ $1${NC}"; exit 1; }
info(){ echo -e "${CYAN}==> $1${NC}"; }

# ── Clean ──────────────────────────────────────────────────────
clean() {
  info "Cleaning..."
  rm -rf "$ROOT/pkg" "$ROOT/build" "$ROOT/src" "$ROOT/*.pkg.tar.*" "$ROOT/*.tar.gz" "$ROOT/*.tar.zst" "$ROOT/*.deb" 2>/dev/null || true
  cargo clean 2>/dev/null || true
  ok "Cleaned"
}

# ── Build Release Binary ───────────────────────────────────────
build_binary() {
  info "Building release binary..."
  cargo build --release
  [ -f "$ROOT/target/release/$NAME" ] || fail "Build failed - no binary found"
  ok "Binary built: target/release/$NAME"
}

# ── Generate GPG Signature ────────────────────────────────────
sign_file() {
  local file="$1"
  if gpg --list-secret-keys --with-colons 2>/dev/null | grep -q "^sec:"; then
    gpg --detach-sign --output "${file}.sig" "$file" 2>/dev/null
    ok "Signed: ${file}.sig"
  else
    warn "No GPG secret key found, skipping signature"
  fi
}

# ── Arch Linux Package (via makepkg) ──────────────────────────
build_arch() {
  info "Building Arch Linux package..."
  command -v makepkg >/dev/null || fail "makepkg not found (install pacman)"

  local srcdir="$ROOT/src"
  rm -rf "$srcdir" "$ROOT/pkg" "$ROOT/*.pkg.tar.*" 2>/dev/null || true

  local tarfile="$ROOT/$NAME-$VERSION.tar.gz"
  [ -f "$tarfile" ] && rm -f "$tarfile"

  mkdir -p "$srcdir/$NAME-$VERSION"
  tar --exclude='.git' --exclude='target' --exclude='pinak.key' \
      --exclude='src' --exclude='build' --exclude='pkg' \
      --exclude='*.tar.gz' --exclude='*.tar.zst' --exclude='*.pkg.tar.*' --exclude='*.deb' \
      -cf - -C "$ROOT" . | tar -xf - -C "$srcdir/$NAME-$VERSION/"
  cd "$srcdir" && tar -czf "$tarfile" "$NAME-$VERSION/" && cd "$ROOT"

  cp "$tarfile" "$ROOT/"
  makepkg -si --noconfirm 2>&1 | tail -5
  rm -f "$ROOT/$NAME-$VERSION.tar.gz"

  local pkgfile="$ROOT/$NAME-$VERSION-1-$PKG_ARCH.pkg.tar.zst"
  [ -f "$pkgfile" ] || fail "Arch package not created"
  sign_file "$pkgfile"
  ok "Arch package: $pkgfile"
}

# ── Debian Package (.deb) ──────────────────────────────────────
build_deb() {
  info "Building Debian package..."
  command -v dpkg-deb >/dev/null || fail "dpkg-deb not found"

  local d="$ROOT/pkg/deb/${NAME}_${VERSION}-1_${DEB_ARCH}"
  rm -rf "$d"
  mkdir -p "$d/DEBIAN" "$d/usr/bin" "$d/usr/share/doc/$NAME" "$d/usr/share/man/man1" \
           "$d/usr/share/licenses/$NAME" "$d/usr/share/bash-completion/completions" \
           "$d/usr/share/zsh/vendor-completions" "$d/usr/share/fish/vendor_completions.d"

  install -Dm755 "$ROOT/target/release/$NAME" "$d/usr/bin/$NAME"
  for f in README.md CONTRIBUTING.md instructions.md; do
    [ -f "$ROOT/$f" ] && install -Dm644 "$ROOT/$f" "$d/usr/share/doc/$NAME/$f"
  done
  [ -f "$ROOT/docs/config.example.toml" ] && install -Dm644 "$ROOT/docs/config.example.toml" "$d/usr/share/doc/$NAME/config.example.toml"
  [ -f "$ROOT/docs/CHANGELOG.md" ] && install -Dm644 "$ROOT/docs/CHANGELOG.md" "$d/usr/share/doc/$NAME/CHANGELOG.md"
  [ -f "$ROOT/docs/RELEASE_NOTES.md" ] && install -Dm644 "$ROOT/docs/RELEASE_NOTES.md" "$d/usr/share/doc/$NAME/RELEASE_NOTES.md"
  [ -f "$ROOT/docs/LICENSE" ] && install -Dm644 "$ROOT/docs/LICENSE" "$d/usr/share/licenses/$NAME/LICENSE"
  [ -f "$ROOT/$NAME.1" ] && install -Dm644 "$ROOT/$NAME.1" "$d/usr/share/man/man1/$NAME.1" && gzip -nf "$d/usr/share/man/man1/$NAME.1" 2>/dev/null || true
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
  sign_file "$debfile"
  ok "Debian package: $debfile"
}

# ── Create GitHub Release ─────────────────────────────────────
create_release() {
  command -v gh >/dev/null || { warn "gh CLI not found, skipping release"; return; }

  local tag="v$VERSION"
  if git rev-parse "$tag" >/dev/null 2>&1; then
    git tag -d "$tag" 2>/dev/null || true
  fi
  git tag "$tag" master
  git push origin "$tag" -f 2>/dev/null || true

  gh release delete "$tag" -y 2>/dev/null || true
  sleep 1

  local files=()
  for f in "$ROOT"/echomind-*.pkg.tar.zst "$ROOT"/echomind_*.deb "$ROOT"/*.sig; do
    [ -f "$f" ] && files+=("$f")
  done

  gh release create "$tag" --title "$tag" --notes "Release $tag" "${files[@]}"
  ok "GitHub release created: $tag"
}

# ── Install Locally ────────────────────────────────────────────
install_local() {
  if command -v pacman &>/dev/null; then
    local pkgfile="$ROOT/$NAME-$VERSION-1-$PKG_ARCH.pkg.tar.zst"
    [ -f "$pkgfile" ] || build_arch
    sudo pacman -U --noconfirm "$ROOT"/"$NAME"-*-"$PKG_ARCH".pkg.tar.zst
  elif command -v dpkg &>/dev/null; then
    local debfile="$ROOT/${NAME}_${VERSION}-1_${DEB_ARCH}.deb"
    [ -f "$debfile" ] || build_deb
    sudo dpkg -i "$ROOT/${NAME}_${VERSION}-1_${DEB_ARCH}.deb"
  else
    build_binary
    sudo cp "$ROOT/target/release/$NAME" /usr/local/bin/
    ok "Installed binary to /usr/local/bin/$NAME"
  fi
}

# ── Help ───────────────────────────────────────────────────────
usage() {
  echo "Usage: $0 {clean|build|arch|deb|release|install|all}"
  echo ""
  echo "  clean    Remove all build artifacts"
  echo "  build    Build release binary only"
  echo "  arch     Build Arch Linux package (.pkg.tar.zst)"
  echo "  deb      Build Debian package (.deb)"
  echo "  release  Build all & upload to GitHub Releases"
  echo "  install  Build & install for your system"
  echo "  all      Build all packages (default)"
  exit 1
}

case "${1:-all}" in
  clean)   clean ;;
  build)   clean; build_binary ;;
  arch)    clean; build_binary; build_arch ;;
  deb)     clean; build_binary; build_deb ;;
  release) clean; build_binary; build_arch; build_deb; create_release ;;
  install) clean; build_binary; install_local ;;
  all)     clean; build_binary; build_arch; build_deb ;;
  *)       usage ;;
esac
