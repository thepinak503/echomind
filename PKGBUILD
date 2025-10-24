# Maintainer: Pinak Dhabu <thepinak503@duck.com>
pkgname=echomind
pkgver=0.1.1
pkgrel=1
pkgdesc="A lightweight CLI tool that pipes input to an AI chat API with coder mode and file output"
arch=('x86_64')
url="https://github.com/thepinak503/echomind"
license=('MIT')
depends=('openssl')
makedepends=('rust' 'cargo' 'git')
optdepends=('jq: for JSON output formatting'
            'curl: for custom API integration')
provides=('echomind')
conflicts=('echomind-git')
source=("git+https://github.com/thepinak503/echomind.git")
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname"
  export RUSTFLAGS="--remap-path-prefix=$(pwd)=."
  cargo build --release
}

check() {
  cd "$srcdir/$pkgname"
  cargo test
}

package() {
  cd "$srcdir/$pkgname"
  install -Dm755 target/release/echomind "$pkgdir/usr/bin/echomind"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 echomind.1 "$pkgdir/usr/share/man/man1/echomind.1"
}