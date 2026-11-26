# Verifying Coincube Releases

This guide explains how to verify the authenticity and integrity of Coincube releases using GPG signatures and SHA256 checksums.

## Why Verify?

Verifying releases ensures:
- **Authenticity**: The release was created by Coincube Technology LLC
- **Integrity**: The files haven't been tampered with during download

## Quick Verification (Recommended)

For each release, we provide:
- `SHA256SUMS-X.Y.Z.txt` - SHA256 checksums of all release artifacts
- `SHA256SUMS-X.Y.Z.txt.asc` - GPG signature of the checksums file

Release artifacts are named `coincube-desktop-<version>-<target>.<ext>`, where `<target>`
is the Rust target triple. For version 1.5.0:

| Platform | Filename |
|----------|----------|
| macOS (Apple Silicon) | `coincube-desktop-1.5.0-aarch64-apple-darwin.dmg` |
| macOS (Intel) | `coincube-desktop-1.5.0-x86_64-apple-darwin.dmg` |
| Windows | `coincube-desktop-1.5.0-x86_64-pc-windows-msvc.msi` |
| Linux | `coincube-desktop-1.5.0-x86_64-unknown-linux-gnu.tar.gz` |

### Step 1: Import the Coincube GPG Public Key (One-Time Setup)

Use the public key for the release you are verifying. The examples below
use the active Desktop key.

| Releases | Public key | Full fingerprint |
|----------|------------|------------------|
| New Desktop releases | [Active key](coincube-release-public.asc) | `B847C6267DD910B90B0DBDB448A152377E8FB081` |
| [Original repository releases](https://github.com/coincubetech/coincube/releases) | [Historical key](release-keys/ED4D0D710103D6252854681367F9701BF0D2DAF4.asc) | `ED4D0D710103D6252854681367F9701BF0D2DAF4` |

For a past release signed with the old key, import the historical key and
download that release's checksum file and detached signature from the original
repository. Check the signature's full fingerprint against the table.

```bash
curl -O https://raw.githubusercontent.com/coincubetech/coincube-desktop/master/docs/security/coincube-release-public.asc
gpg --import coincube-release-public.asc
```

Expected output should include:
```bash
gpg: key 48A152377E8FB081: public key "Coincube Release Signing <releases@coincube.io>" imported
```

### Step 2: Download Release Files

Download the artifact you want to install plus the checksums files:

```bash
# Example for macOS Apple Silicon (arm64), version 1.5.0
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v1.5.0/coincube-desktop-1.5.0-aarch64-apple-darwin.dmg
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v1.5.0/SHA256SUMS-1.5.0.txt
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v1.5.0/SHA256SUMS-1.5.0.txt.asc
```

### Step 3: Verify the GPG Signature

```bash
gpg --verify SHA256SUMS-1.5.0.txt.asc
```

Expected output:
```bash
gpg: assuming signed data in 'SHA256SUMS-1.5.0.txt'
gpg: Signature made [DATE]
gpg:                using RSA key B847C6267DD910B90B0DBDB448A152377E8FB081
gpg: Good signature from "Coincube Release Signing <releases@coincube.io>"
```

⚠️ **Warning**: If you see `BAD signature`, do NOT proceed. The checksums file has been tampered with.

### Step 4: Verify the Artifact Checksum

```bash
sha256sum --check SHA256SUMS-1.5.0.txt --ignore-missing
```

Expected output:
```bash
coincube-desktop-1.5.0-aarch64-apple-darwin.dmg: OK
```

✅ If both verifications pass, your download is authentic and safe to install.

## Platform-Specific Examples

### macOS

```bash
# Import key (one-time)
curl -O https://raw.githubusercontent.com/coincubetech/coincube-desktop/master/docs/security/coincube-release-public.asc
gpg --import coincube-release-public.asc

# Download files
VERSION=1.5.0
TARGET=aarch64-apple-darwin  # or x86_64-apple-darwin for Intel Macs
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v${VERSION}/coincube-desktop-${VERSION}-${TARGET}.dmg
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v${VERSION}/SHA256SUMS-${VERSION}.txt
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v${VERSION}/SHA256SUMS-${VERSION}.txt.asc

# Verify
gpg --verify SHA256SUMS-${VERSION}.txt.asc
shasum -a 256 --check SHA256SUMS-${VERSION}.txt --ignore-missing
```

### Linux

```bash
# Import key (one-time)
curl -O https://raw.githubusercontent.com/coincubetech/coincube-desktop/master/docs/security/coincube-release-public.asc
gpg --import coincube-release-public.asc

# Download files
VERSION=1.5.0
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v${VERSION}/coincube-desktop-${VERSION}-x86_64-unknown-linux-gnu.tar.gz
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v${VERSION}/SHA256SUMS-${VERSION}.txt
curl -LO https://github.com/coincubetech/coincube-desktop/releases/download/v${VERSION}/SHA256SUMS-${VERSION}.txt.asc

# Verify
gpg --verify SHA256SUMS-${VERSION}.txt.asc
sha256sum --check SHA256SUMS-${VERSION}.txt --ignore-missing
```

### Windows (PowerShell)

```powershell
# Import key (one-time)
# Install GPG4Win first: https://gpg4win.org/
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/coincubetech/coincube-desktop/master/docs/security/coincube-release-public.asc" -OutFile "coincube-release-public.asc"
gpg --import coincube-release-public.asc

# Download files
$VERSION = "1.5.0"
Invoke-WebRequest -Uri "https://github.com/coincubetech/coincube-desktop/releases/download/v$VERSION/coincube-desktop-$VERSION-x86_64-pc-windows-msvc.msi" -OutFile "coincube-desktop-$VERSION-x86_64-pc-windows-msvc.msi"
Invoke-WebRequest -Uri "https://github.com/coincubetech/coincube-desktop/releases/download/v$VERSION/SHA256SUMS-$VERSION.txt" -OutFile "SHA256SUMS-$VERSION.txt"
Invoke-WebRequest -Uri "https://github.com/coincubetech/coincube-desktop/releases/download/v$VERSION/SHA256SUMS-$VERSION.txt.asc" -OutFile "SHA256SUMS-$VERSION.txt.asc"

# Verify signature
gpg --verify "SHA256SUMS-$VERSION.txt.asc"

# Verify checksum (manual check)
Get-FileHash -Algorithm SHA256 coincube-desktop-$VERSION-x86_64-pc-windows-msvc.msi
# Compare output with the hash in SHA256SUMS-$VERSION.txt
```

## Troubleshooting

### "gpg: command not found"

Install GPG:
- **macOS**: `brew install gnupg`
- **Linux**: `sudo apt-get install gnupg` (Debian/Ubuntu) or `sudo yum install gnupg` (RHEL/CentOS)
- **Windows**: Download from [GPG4Win](https://gpg4win.org/)

### "WARNING: This key is not certified with a trusted signature"

GPG can report this warning for a valid signature when the signing key has
no assigned trust in your keyring. Confirm that the signature's full
fingerprint matches the entry for that release in the key table above.

### "No such file or directory"

Ensure you're in the directory containing the downloaded files, or provide full paths.

## GPG Key Information

| Property | Value |
|----------|-------|
| Active key fingerprint | `B847C6267DD910B90B0DBDB448A152377E8FB081` |
| Email | [releases@coincube.io](mailto:releases@coincube.io) |
| Name | Coincube Release Signing |
| Type | RSA 4096 |
| Usage | Sign only |

## Security Notes

- Always verify both the GPG signature AND the checksum
- Download the public key from the official repository or website
- Never skip verification, especially for financial software
- Report any verification failures to [security@coincube.io](mailto:security@coincube.io)

## Additional Resources

- [GPG Key Rotation Playbook](./GPG_KEY_ROTATION.md)
