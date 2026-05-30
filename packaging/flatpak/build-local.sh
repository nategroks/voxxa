#!/usr/bin/env bash
# Build Voxxa as a Flatpak locally. Requires flatpak + flatpak-builder.
#
# Usage: ./packaging/flatpak/build-local.sh [--install]
#
# Without --install, produces a .flatpak bundle in ./build/voxxa.flatpak.
# With --install, also installs into the user's flatpak repository.
set -euo pipefail

cd "$(dirname "$0")/../.."

# Required runtimes + SDKs. Pulled from flathub if not already present.
flatpak install --user --noninteractive flathub \
  org.freedesktop.Platform//24.08 \
  org.freedesktop.Sdk//24.08 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.node20//24.08 \
  || true

BUILD_DIR="build/flatpak"
REPO_DIR="build/flatpak-repo"
rm -rf "$BUILD_DIR" "$REPO_DIR"
mkdir -p "$BUILD_DIR" "$REPO_DIR"

flatpak-builder \
  --force-clean \
  --user \
  --install-deps-from=flathub \
  --repo="$REPO_DIR" \
  "$BUILD_DIR" \
  packaging/flatpak/com.voxxa.app.yml

mkdir -p build
flatpak build-bundle "$REPO_DIR" build/voxxa.flatpak com.voxxa.app
echo "Built: build/voxxa.flatpak"

if [[ "${1:-}" == "--install" ]]; then
  flatpak install --user --reinstall --assumeyes build/voxxa.flatpak
  echo "Installed. Launch with: flatpak run com.voxxa.app"
fi
