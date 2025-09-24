#!/bin/bash
if [ ! -f "PKGBUILD" ]; then
    echo "Error: PKGBUILD does not exist"
    exit 1
fi

if [ ! -f "PKGBUILD-BIN" ]; then
    echo "Error: PKGBUILD-BIN does not exist"
    exit 1
fi

VERSION=$(grep "^pkgver=" PKGBUILD | cut -d'=' -f2)
if [ -z "$VERSION" ]; then
    echo "Error: Impossible to extract the version from PKGBUILD"
    exit 1
fi

echo "Version found: $VERSION"

sed -i "s/^pkgver=.*/pkgver=$VERSION/" PKGBUILD-BIN

echo "Version updated in PKGBUILD-BIN: $VERSION"
