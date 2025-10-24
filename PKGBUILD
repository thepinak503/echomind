# Maintainer: Pinak Dhabu <thepinak503@duck.com>
pkgname=echomind
pkgver=0.1.1
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
source=("$pkgname-$pkgver.tar.gz::https://github.com/thepinak503/echomind/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('ab89ec302baa9581c2fd8b23658ca88f494fc42be16c8760772754a6910bbabf')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  export RUSTFLAGS="--remap-path-prefix=$(pwd)=."
  cargo build --release
}

check() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo test
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 target/release/echomind "$pkgdir/usr/bin/echomind"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 echomind.1 "$pkgdir/usr/share/man/man1/echomind.1"
}