# Snap Package Publishing Guide

This document explains how to publish and update the Scrivano snap package to the Ubuntu Snap Store.

## Prerequisites

1. **Ubuntu One Account**: You need an Ubuntu One account to publish snaps
2. **Snapcraft CLI**: Install snapcraft
   ```bash
   sudo snap install snapcraft --classic
   ```
3. **Developer Registration**: Register as a snap developer
   ```bash
   snapcraft login
   ```

## First-Time Setup

### 1. Register the Snap Name

```bash
snapcraft register scrivano
```

This reserves the name "scrivano" in the Snap Store. You only need to do this once.

### 2. Build Configuration

The snap configuration is in `snap/snapcraft.yaml`. Key features:

- **Models**: Whisper models are bundled in the snap and copied to `$SNAP_USER_DATA/models` on first run
- **Version**: Automatically synced with Cargo.toml version
- **Confinement**: Uses `strict` for store compatibility without manual Canonical approval
- **Icon**: Located at `snap/gui/icon.png` (256x256 PNG)
- **Screenshots**: Located at `screenshots/` directory

## Building the Snap

### Step1: Build the Release Binary

```bash
# Build release binary
cargo build --release --features "audio-playback tray-icon"

# Create snap directory structure
mkdir -p snap/bin

# Copy binary to snap directory
cp target/release/scrivano snap/bin/
```

### Step 2: Build the Snap Package

```bash
cd snap
snapcraft
```

This creates a`.snap` file: `scrivano_1.1.8_amd64.snap`

### Step3: Test Locally (Optional)

```bash
# Install locally for testing
sudo snap install --dangerous scrivano_1.1.8_amd64.snap

# Test the application
scrivano

# Uninstall when done
sudo snap remove scrivano
```

## Publishing to Snap Store

### Publishing to Stable Channel

```bash
# Upload and release to stable channel
snapcraft upload --release=stable scrivano_1.1.8_amd64.snap
```

### Publishing to Other Channels

```bash
# Edge channel (for testing/development)
snapcraft upload --release=edge scrivano_1.1.8_amd64.snap

# Beta channel (for beta testing)
snapcraft upload --release=beta scrivano_1.1.8_amd64.snap

# Candidate channel (for release candidates)
snapcraft upload --release=candidate scrivano_1.1.8_amd64.snap
```

## Updating the Snap

### Version Update Workflow

1. **Update version in Cargo.toml**:
   ```toml
   version = "1.1.9"
   ```

2. **Update version in snap/snapcraft.yaml**:
   ```yaml
   version: '1.1.9'
   ```

3. **Build new release binary**:
   ```bash
   cargo build --release --features "audio-playback tray-icon"
   cp target/release/scrivano snap/bin/
   ```

4. **Build snap**:
   ```bash
   cd snap
   snapcraft
   ```

5. **Upload to store**:
   ```bash
   snapcraft upload --release=stable scrivano_1.1.9_amd64.snap
   ```

### Managing Channels

```bash
# Promote from beta to stable
snapcraft release scrivano <revision> stable

# List all revisions
snapcraft list-revisions scrivano

# View channel status
snapcraft status scrivano
```

## Adding Screenshots

Screenshots are uploaded separately through the Snap Store dashboard:

1. Go to https://dashboard.snapcraft.io/snaps/scrivano/
2. Navigate to "Listing" tab
3. Upload screenshots from `screenshots/` directory
4. Screenshots should be:
   - PNG format
   - At least 640x480 pixels
   - Maximum 2560x1920 pixels
   - No more than 5 screenshots

## Model Handling Behavior

The snap uses a wrapper script (`snap/bin/scrivano-wrapper`) that:

1. Creates `$SNAP_USER_DATA/models/` directory
2. Copies bundled models on first run:
   - `ggml-tiny.bin` (77 MB)
   - `ggml-small-q5_1.bin` (190 MB)
3. Reuses existing models if they are already present
4. Launches the main application

## Snap Configuration Details

### Architecture Support

The snap supports both x86_64 (amd64) and ARM64:

```yaml
architectures:
  - amd64
  - arm64
```

To build for different architectures:

```bash
# Build for AMD64
snapcraft --target-arch=amd64

# Build for ARM64
snapcraft --target-arch=arm64
```

### Permissions (Plugs)

```yaml
plugs:
  - audio-playback      # Play audio
  - network             # Ollama integration, optional network access
  - home                # Access user home directory
  - removable-media     # Export recordings to USB drives
```

## Troubleshooting

### Build Errors

```bash
# Clean snap build artifacts
snapcraft clean

# Rebuild from scratch
snapcraft
```

### Model Availability Issues

If models are missing:
- Check bundled files inside snap: `unsquashfs -l snap/scrivano_1.1.8_amd64.snap | grep models/`
- Check user models path: `ls -la $HOME/snap/scrivano/current/models/`

### Runtime Issues

```bash
# View snap logs
journalctl -u snap.scrivano

# Check snap confinement
snap info scrivano

# Verify plugs are connected
snap connections scrivano
```

## Automation (CI/CD)

For automated releases via GitHub Actions, see `.github/workflows/build-release.yml`.

The workflow should:
1. Build release binary
2. Copy to snap/bin/
3. Run snapcraft
4. Upload to store (requires`SNAPCRAFT_CREDENTIALS` secret)

## Resources

- [Snapcraft Documentation](https://snapcraft.io/docs)
- [Snapcraft YAML Reference](https://snapcraft.io/docs/snapcraft-yaml-reference)
- [Snap Store Dashboard](https://dashboard.snapcraft.io/)
- [Debugging Snaps](https://snapcraft.io/docs/debugging-snaps)
