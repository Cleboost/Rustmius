pkgname=rustmius-bin
_pkgname=rustmius
pkgver=2.0.0
pkgrel=1
pkgdesc="Une alternative locale complète à Termius pour Linux (GTK4)"
arch=('x86_64' 'aarch64')
url="https://github.com/Cleboost/Rustmius"
license=('MIT')
depends=('libadwaita' 'gtk4' 'vte4')
provides=("$_pkgname")
conflicts=("$_pkgname")

source_x86_64=("$url/releases/download/v$pkgver/$_pkgname-x86_64")
source_aarch64=("$url/releases/download/v$pkgver/$_pkgname-aarch64")

sha256sums_x86_64=('SKIP')
sha256sums_aarch64=('SKIP')

package() {
    if [ "$CARCH" == "x86_64" ]; then
        local _bin="$_pkgname-x86_64"
    else
        local _bin="$_pkgname-aarch64"
    fi

    install -Dm755 "$_bin" "$pkgdir/usr/bin/$_pkgname"
    install -Dm644 "$_pkgname.desktop" "$pkgdir/usr/share/applications/$_pkgname.desktop"
    install -Dm644 "$_pkgname.png" "$pkgdir/usr/share/icons/hicolor/512x512/apps/$_pkgname.png"
}
