# Maintainer: Pinak Dhabu <thepinak503@duck.com>
pkgname=echomind
pkgver=0.2.0
pkgrel=1
pkgdesc="A lightweight CLI tool that pipes input to an AI chat API with coder mode and file output"
arch=('x86_64')
url="https://github.com/thepinak503/echomind"
license=('MIT')
depends=('openssl')
makedepends=('rust' 'cargo')
optdepends=('jq: for JSON output formatting'
            'curl: for custom API integration')
provides=('echomind')
conflicts=('echomind-git')

build() {
  export RUSTFLAGS="--remap-path-prefix=$(pwd)=."
  cargo clean
  cargo build --release
}

package() {
  install -Dm755 "$(pwd)/target/release/echomind" "$pkgdir/usr/bin/echomind"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 echomind.1 "$pkgdir/usr/share/man/man1/echomind.1"
}