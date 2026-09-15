#!/usr/bin/env bash
#
# Build a .rpm package for Rustmius (Fedora / RHEL / openSUSE family).
#
# Usage:
#   packages/rpm/build-rpm.sh               # builds release binary if missing, then packages
#   SKIP_BUILD=1 packages/rpm/build-rpm.sh   # reuse an existing target/release/rustmius
#
# Output: dist/rustmius-<version>-<release>.<arch>.rpm (in the repo root)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

PKG_NAME="rustmius"
VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')"

command -v rpmbuild >/dev/null 2>&1 || {
    echo "!! rpmbuild not found. Install it with: sudo dnf install rpm-build rpmdevtools" >&2
    exit 1
}

echo ">> Packaging $PKG_NAME $VERSION (.rpm)"

# Private rpmbuild tree so this never touches ~/rpmbuild.
TOPDIR="$(mktemp -d)"
STAGE="$(mktemp -d)"
trap 'rm -rf "$TOPDIR" "$STAGE"' EXIT
mkdir -p "$TOPDIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Snapshot the working tree (including uncommitted edits, like build-deb.sh
# building straight from target/release) into a source tarball. Name matches
# the spec's %autosetup -n rustmius-$VERSION.
SRC_DIR="$STAGE/$PKG_NAME-$VERSION"
mkdir -p "$SRC_DIR"
tar --exclude=.git --exclude=target --exclude=dist -cf - -- * | tar -xf - -C "$SRC_DIR"

# With SKIP_BUILD=1, carry the already-built binary into the tarball so the
# spec's %build step can skip its own `cargo build` (mirrors build-deb.sh).
if [[ "${SKIP_BUILD:-0}" == "1" && -x "target/release/$PKG_NAME" ]]; then
    echo ">> Reusing prebuilt target/release/$PKG_NAME (SKIP_BUILD=1)"
    install -Dm755 "target/release/$PKG_NAME" "$SRC_DIR/target/release/$PKG_NAME"
fi

tar -czf "$TOPDIR/SOURCES/$PKG_NAME-$VERSION.tar.gz" -C "$STAGE" "$PKG_NAME-$VERSION"

cp "packages/rpm/$PKG_NAME.spec" "$TOPDIR/SPECS/"

SKIP_BUILD="${SKIP_BUILD:-0}" rpmbuild --define "_topdir $TOPDIR" -ba "$TOPDIR/SPECS/$PKG_NAME.spec"

mkdir -p dist
find "$TOPDIR/RPMS" -name '*.rpm' -exec cp -v {} dist/ \;
find "$TOPDIR/SRPMS" -name '*.rpm' -exec cp -v {} dist/ \;

echo ""
echo ">> Built RPM(s) in dist/:"
ls -1 dist/*.rpm
