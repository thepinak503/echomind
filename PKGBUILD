# Maintainer: Pinak Dhabu <thepinak503@duck.com>
pkgname=echomind
pkgver=0.1.1
pkgrel=1
pkgdesc="A simple tool to send piped input to a chat API and print the response"
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
sha256sums=('cd3e83535fc8f51552cc4013ed10a7c01610cbdccee50032841ecf388eaa1678')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  export RUSTFLAGS="--remap-path-prefix=$(pwd)=."
  cargo build --release --locked
}

check() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo test --locked
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 target/release/echomind "$pkgdir/usr/bin/echomind"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 echomind.1 "$pkgdir/usr/share/man/man1/echomind.1"
}