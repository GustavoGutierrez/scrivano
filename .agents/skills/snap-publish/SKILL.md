# Snap Publish Skill

## Purpose
Publish Scrivano snap package to Ubuntu Snap Store.

## Prerequisites
- snapcraft installed (`sudo snap install snapcraft --classic`)
- Logged in to Snap Store (`snapcraft login`)
- Registered snap name (`snapcraft register scrivano`)

## Workflow

### 1. Prepare binary
Build or download the Linux x86_64 binary:
- From local: `cargo build --release`
- From GitHub Actions artifacts: download `scrivano-linux-x86_64`

### 2. Prepare icons (already in repo)
Icons are located in `snap/gui/`:
```
snap/gui/
├── icon.png (512x512)          # Main icon for snap store
├── favicon-256x256.png         # High-res
├── favicon-128x128.png         # App grid
├── favicon-64x64.png           # System tray
├── favicon-48x48.png           # Alt size
├── favicon-32x32.png           # Small
└── favicon-16x16.png           # Tiny
```
These are already configured in `snap/snapcraft.yaml`.

### 3. Create snap package
```bash
# Copy binary to snap/bin/
mkdir -p snap/bin
cp /path/to/scrivano-linux-x86_64 snap/bin/scrivano
chmod +x snap/bin/scrivano

# Build snap
cd snap
snapcraft clean
snapcraft

# Or pack directly
snapcraft pack
```

### 4. Upload to Snap Store
```bash
# Upload to stable channel
snapcraft upload --release=stable scrivano_1.0.2_amd64.snap

# Or test in candidate first
snapcraft upload --release=candidate scrivano_1.0.2_amd64.snap
```

### 5. Publish and track
```bash
# View published snap
snapcraft list-revisions scrivano

# Track metrics
snapcraft metrics scrivano
```

## Snap Metadata

| Field | Value |
|-------|-------|
| Name | scrivano |
| Developer | Gustavo Gutiérrez |
| License | MIT |
| Homepage | https://github.com/GustavoGutierrez/scrivano |
| Icon | gui/icon.png |

## Plugs (Permissions)
- audio-playback
- network
- network-client
- home
- removable-media

## Release Process

1. Update version in `snap/snapcraft.yaml` (already done: 1.0.2)
2. Download Linux binary from GitHub Release
3. Copy to snap/bin/scrivano
4. Run `snapcraft pack`
5. Upload with `snapcraft upload --release=stable`

## Troubleshooting

### Icon not showing
- Ensure icon.png is 512x512 minimum
- Verify icon path in snapcraft.yaml: `icon: gui/icon.png`
- Use `snapcraft audit` to check for issues

### Permission denied
- Ensure binary is executable: `chmod +x snap/bin/scrivano`
- confinement: classic allows most operations

### Build fails
- Clean build: `snapcraft clean`
- Use LXD container: `snapcraft pack --use-lxd`