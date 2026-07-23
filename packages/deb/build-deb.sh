#!/usr/bin/env bash
#
# Build a .deb package for Rustmius (Debian / Ubuntu).
#
# Usage:
#   packages/deb/build-deb.sh            # builds release binary if missing, then packages
#   SKIP_BUILD=1 packages/deb/build-deb.sh   # reuse an existing target/release/rustmius
#
# Output: dist/rustmius_<version>_<arch>.deb (in the repo root)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

PKG_NAME="rustmius"
VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')"
MAINTAINER="${DEB_MAINTAINER:-Clement Balarot <clement.balarot@gmail.com>}"
DEB_ARCH="$(dpkg --print-architecture)"

echo ">> Packaging $PKG_NAME $VERSION for $DEB_ARCH"

# Build the release binary.
BIN="target/release/$PKG_NAME"
if [[ "${SKIP_BUILD:-0}" != "1" || ! -x "$BIN" ]]; then
    echo ">> Building release binary (fat LTO, this can take a while)..."
    cargo build --release --locked
fi
[[ -x "$BIN" ]] || { echo "!! Binary not found at $BIN" >&2; exit 1; }

# Assemble the package tree.
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT
chmod 755 "$STAGE"

install -Dm755 "$BIN" "$STAGE/usr/bin/$PKG_NAME"
install -Dm644 "packages/org.rustmius.Rustmius.desktop" \
    "$STAGE/usr/share/applications/org.rustmius.Rustmius.desktop"

# Icon: resize to 512x512 when ImageMagick is available, else keep the source
# 500x500 in a size-matched hicolor dir.
ICON_SRC="packages/rustmius.png"
if command -v convert >/dev/null 2>&1; then
    install -dm755 "$STAGE/usr/share/icons/hicolor/512x512/apps"
    convert "$ICON_SRC" -resize 512x512 \
        "$STAGE/usr/share/icons/hicolor/512x512/apps/$PKG_NAME.png"
elif command -v magick >/dev/null 2>&1; then
    install -dm755 "$STAGE/usr/share/icons/hicolor/512x512/apps"
    magick "$ICON_SRC" -resize 512x512 \
        "$STAGE/usr/share/icons/hicolor/512x512/apps/$PKG_NAME.png"
else
    install -Dm644 "$ICON_SRC" \
        "$STAGE/usr/share/icons/hicolor/500x500/apps/$PKG_NAME.png"
fi

find "$STAGE/usr/share/icons" -type f -name '*.png' -exec chmod 644 {} +

# Man page (version stamped in from Cargo.toml).
install -dm755 "$STAGE/usr/share/man/man1"
sed "s/@VERSION@/$VERSION/g" "packages/deb/rustmius.1" \
    > "$STAGE/usr/share/man/man1/$PKG_NAME.1"
chmod 644 "$STAGE/usr/share/man/man1/$PKG_NAME.1"
gzip -9n "$STAGE/usr/share/man/man1/$PKG_NAME.1"

# Copyright file + compressed changelog under /usr/share/doc (Debian policy).
# Native package, so the changelog is named "changelog.gz".
install -Dm644 "packages/deb/copyright" "$STAGE/usr/share/doc/$PKG_NAME/copyright"
CHANGELOG="$STAGE/usr/share/doc/$PKG_NAME/changelog"
cat > "$CHANGELOG" <<EOF
rustmius ($VERSION) unstable; urgency=medium

  * Release $VERSION. See the GitHub release notes for details:
    https://github.com/Cleboost/Rustmius/releases/tag/v$VERSION

 -- $MAINTAINER  $(date -R)
EOF
chmod 644 "$CHANGELOG"
gzip -9n "$CHANGELOG"

# Compute dependencies from the actual binary via dpkg-shlibdeps.
DEPENDS=""
if command -v dpkg-shlibdeps >/dev/null 2>&1; then
    echo ">> Resolving shared-library dependencies..."
    SHLIB_DIR="$(mktemp -d)"
    mkdir -p "$SHLIB_DIR/debian"
    touch "$SHLIB_DIR/debian/control"
    ( cd "$SHLIB_DIR" && dpkg-shlibdeps -O "$REPO_ROOT/$BIN" 2>/dev/null ) \
        > "$SHLIB_DIR/out" || true
    DEPENDS="$(sed -E 's/^shlibs:Depends=//' "$SHLIB_DIR/out" | head -n1)"
    rm -rf "$SHLIB_DIR"
fi
# Fallback if shlibdeps is unavailable.
if [[ -z "$DEPENDS" ]]; then
    DEPENDS="libc6, libgtk-4-1 (>= 4.6), libvte-2.91-gtk4-0, libssl3"
fi
echo ">> Depends: $DEPENDS"

# Control file.
INSTALLED_SIZE="$(du -k -s "$STAGE" | cut -f1)"
mkdir -p "$STAGE/DEBIAN"

cat > "$STAGE/DEBIAN/control" <<EOF
Package: $PKG_NAME
Version: $VERSION
Architecture: $DEB_ARCH
Maintainer: $MAINTAINER
Installed-Size: $INSTALLED_SIZE
Depends: $DEPENDS
Section: net
Priority: optional
Homepage: https://github.com/Cleboost/Rustmius
Description: Local Termius alternative for Linux (GTK4)
 Rustmius is a modern, fast, and fully local alternative to Termius,
 built with Rust and GTK4. It provides an integrated SSH terminal
 (via VTE), an advanced SFTP explorer with drag & drop, a host
 manager, and secure secret storage through the system keyring.
EOF

# Maintainer scripts: refresh icon + desktop caches.
cat > "$STAGE/DEBIAN/postinst" <<'EOF'
#!/bin/sh
set -e
if [ "$1" = "configure" ]; then
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
fi
EOF
cat > "$STAGE/DEBIAN/postrm" <<'EOF'
#!/bin/sh
set -e
if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
fi
EOF
chmod 755 "$STAGE/DEBIAN/postinst" "$STAGE/DEBIAN/postrm"

# Build the .deb.
mkdir -p dist
OUT="dist/${PKG_NAME}_${VERSION}_${DEB_ARCH}.deb"
dpkg-deb --root-owner-group --build "$STAGE" "$OUT"

echo ""
echo ">> Built: $OUT"
dpkg-deb --info "$OUT"
echo ">> Contents:"
dpkg-deb --contents "$OUT"
