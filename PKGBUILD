pkgname=rustmius
pkgdisplayname="Rustmius"
pkgver=0.1.0
pkgrel=1
pkgdesc="Full local Termius alternative for Linux"
arch=('x86_64')
url="https://github.com/Cleboost/Rustmius"
license=('MIT')
depends=('rust' 'cargo' 'pkg-config' 'libadwaita' 'gtk4')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")

prepare() {
    cd "Rustmius-$pkgver"
    cargo build --release
}

package() {
    cd "Rustmius-$pkgver"
    install -Dm755 target/release/$pkgname "$pkgdir/usr/bin/$pkgname"
    install -Dm644 $pkgname.desktop "$pkgdir/usr/share/applications/$pkgname.desktop"
    install -Dm644 $pkgname.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/$pkgname.png"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    #install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}