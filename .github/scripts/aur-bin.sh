#!/bin/bash
cp PKGBUILD PKGBUILD-BIN


sed -i 's/pkgname=rustmius/pkgname=rustmius-bin/g' PKGBUILD-BIN
sed -i 's/depends=('\''rust'\'' '\''cargo'\'' '\''pkg-config'\'' '\''libadwaita'\'' '\''gtk4'\'')/depends=('\''libadwaita'\'' '\''gtk4'\'')/g' PKGBUILD-BIN
#sed -i 's|source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")|source=("$pkgname-$pkgver::$url/releases/download/v$pkgver/rustmius" "rustmius.desktop::$url/raw/v$pkgver/rustmius.desktop" "rustmius.png::$url/raw/v$pkgver/rustmius.png")|g' PKGBUILD-BIN
sed -i 's|source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")|source=("$pkgname-$pkgver::$url/releases/download/v$pkgver/rustmius" "rustmius.desktop::$url/raw/v$pkgver/rustmius.desktop" "README.md::$url/raw/v$pkgver/README.md")|g' PKGBUILD-BIN
sed -i 's/sha256sums=('\''SKIP'\'')/sha256sums=('\''SKIP'\'' '\''SKIP'\'' '\''SKIP'\'')/g' PKGBUILD-BIN


sed -i '/^prepare()/,/^}/d' PKGBUILD-BIN
sed -i '/^package()/,/^}/d' PKGBUILD-BIN

cat >> PKGBUILD-BIN << 'EOF'
package() {
    install -Dm755 "$pkgname-$pkgver" "$pkgdir/usr/bin/rustmius"
    install -Dm644 rustmius.desktop "$pkgdir/usr/share/applications/rustmius.desktop"
    #install -Dm644 rustmius.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/rustmius.png"
    install -Dm644 README.md "$pkgdir/usr/share/doc/rustmius/README.md"
}
EOF
