# Maintainer: [Your Name] <[your email]>
pkgname=echomind
pkgver=0.1.0
pkgrel=1
pkgdesc="A simple tool to send piped input to a chat API and print the response"
arch=('x86_64')
url="https://github.com/thepinak503/echomind"
license=('MIT')
depends=('openssl')
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/thepinak503/echomind/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 target/release/echomind "$pkgdir/usr/bin/echomind"
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 echomind.1 "$pkgdir/usr/share/man/man1/echomind.1"
}