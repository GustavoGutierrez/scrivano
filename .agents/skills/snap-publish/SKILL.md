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
- From GitHub Actions artifacts

### 2. Create snap package
```bash
# Copy binary to snap/bin/
mkdir -p snap/bin
cp target/release/scrivano snap/bin/

# Build snap
cd snap
snapcraft

# Or pack directly
snapcraft pack
```

### 3. Upload to Snap Store
```bash
# Upload to stable channel
snapcraft upload --release=stable scrivano_1.0.0_amd64.snap

# Or test in candidate first
snapcraft upload --release=candidate scrivano_1.0.0_amd64.snap
```

### 4. Publish and track
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

## Plugs (Permissions)
- audio-playback
- network
- network-client
- home
- removable-media

## Release Process

1. Update version in `snap/snapcraft.yaml`
2. Build release binary
3. Copy to snap/bin/
4. Run `snapcraft pack`
5. Upload with `snapcraft upload --release=stable`