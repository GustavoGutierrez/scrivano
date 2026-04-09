#!/bin/bash
# Build script for Scrivano Snap package
# This script builds the release binary and snap package

set -e

VERSION=$(grep -E '^version = ' Cargo.toml | cut -d'"' -f2)
echo "========================================="
echo "Building Scrivano v${VERSION} Snap Package"
echo "========================================="

# Step 1: Build release binary
echo ""
echo "[1/4] Building release binary..."
cargo build --release --features "audio-playback tray-icon"

# Step 2: Prepare snap directory
echo ""
echo "[2/4] Preparing snap directory..."
mkdir -p snap/dist-bin
mkdir -p snap/models
mkdir -p snap/wrapper
cp target/release/scrivano snap/dist-bin/
chmod +x snap/wrapper/scrivano-wrapper
cp models/ggml-tiny.bin snap/models/
cp models/ggml-small-q5_1.bin snap/models/

# Step 3: Build snap in destructive mode (builds on host)
echo ""
echo "[3/4] Building snap package (destructive mode)..."
cd snap
snapcraft clean
snapcraft pack --destructive-mode

# Step 4: Show results
echo ""
echo "[4/4] Build complete!"
echo ""
echo "✓ Snap package created successfully:"
ls -lh scrivano_*.snap

# Instructions for publishing
echo ""
echo "========================================="
echo "Next Steps"
echo "========================================="
echo ""
echo "To test locally:"
echo "  sudo snap install --dangerous scrivano_${VERSION}_amd64.snap"
echo "  scrivano"
echo ""
echo "To publish to Snap Store:"
echo "  1. snapcraft login"
echo "  2. snapcraft register scrivano  # Only first time"
echo "  3. snapcraft upload --release=stable scrivano_${VERSION}_amd64.snap"
echo "  4. Add screenshots at: https://dashboard.snapcraft.io/snaps/scrivano/"
echo ""
echo "Documentation:"
echo "  - Publishing guide: docs/snap-publishing-guide.md"
echo "  - Quick steps: docs/snap-publishing-steps.md"
echo ""
