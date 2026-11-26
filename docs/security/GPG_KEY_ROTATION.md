# GPG Release Key Rotation Playbook  
## COINCUBE TECHNOLOGY LLC

## Purpose

This document defines the safe, repeatable process for rotating the Coincube GPG release signing key used to cryptographically sign all release artifacts (macOS, Windows, Linux).

GPG signing provides vendor-level trust independent of platform vendors (Apple, Microsoft, Linux distros).

---

## GPG Key in Scope

| Item | Value |
|----|-----|
| Key Name | Coincube Release Signing |
| Email | releases@coincube.io |
| Usage | Sign-only |
| Key Size | RSA 4096 |
| CI Storage | GitHub Actions Secrets |
| Artifacts Signed | DMG, MSI, Linux tarballs |

---

## When to Rotate

### Scheduled Rotation
- Every 24 months

### Immediate Rotation Required If
- Private key is suspected compromised
- GitHub Secrets leak is suspected
- Key expiration is approaching
- GPG verification failures are reported by users

---

## Rotation Overview

1. Generate the replacement key and export its public and private material.
2. Restore-test the protected backup and verify a signature from the new key.
3. Stage the GitHub secrets.
4. Publish the new public key and update the workflow fingerprints.
5. Enable release signing with the replacement.

Keep each retired public key under `docs/security/release-keys/` with its full
fingerprint as the filename. The existing release key is preserved byte for
byte in [the release key archive](release-keys/ED4D0D710103D6252854681367F9701BF0D2DAF4.asc).

Before activating the replacement key, record the first release version that
uses it. Update the active public key and both `expected_fingerprint` checks
in `.github/workflows/releases.yml` together. Update the verification guide and
keep the archived key available for releases from before the cutoff.

---

## Step-by-Step Rotation Procedure

### 1. Generate a New GPG Key

On a secure, trusted machine:

```bash
gpg --full-generate-key
```

Recommended settings:
- Type: RSA
- Size: 4096
- Usage: Sign only
- Name: Coincube Release Signing
- Email: releases@coincube.io
- Expiration: 2 years
- Passphrase: Strong and unique

List key ID:

```bash
gpg --list-secret-keys --keyid-format LONG
```

---

### 2. Export Keys

```bash
gpg --export-secret-keys --armor <KEY_ID> > coincube-release-private.asc
gpg --export --armor <KEY_ID> > coincube-release-public.asc
```

Store the private key securely.

---

### 3. Verify the Replacement

Restore the encrypted backup into an isolated GPG keyring. Use it to sign a
sample checksum file and verify the signature in a second keyring that holds
only the replacement public key. Check the full signature fingerprint. Use a
historical checksum and signature to confirm that the archived public key
still works for historical verification.

---

### 4. Stage GitHub Secrets

Keep production release signing paused while the secrets and workflow
fingerprints are being updated. Set both secrets in the standalone repo.

| Secret | Description |
|--------|-------------|
| GPG_PRIVATE_KEY | New private key (ASCII armored) |
| GPG_PASSPHRASE | Passphrase for the key |

The tag-triggered production workflow publishes release assets. Use an
owner-approved release for the first live verification after reviewing and
merging the public key and both workflow fingerprints.

---

### 5. Publish the New Public Key

Publish the public key to:
- docs/security/coincube-release-public.asc
- GitHub Releases
- Coincube website (recommended)

Optional keyserver publish:

```bash
gpg --send-keys <KEY_ID>
```

---

### 6. Deprecate Old Key

- Keep old public key available
- Document cutoff release version
- Optionally revoke old key:

```bash
gpg --gen-revoke <OLD_KEY_ID>
```

Publish revocation certificate if generated.

---

## User Verification Example

Users verify releases using the checksums file:

```bash
# Import the public key (one-time setup)
gpg --import coincube-release-public.asc

# Verify the checksums file signature
gpg --verify SHA256SUMS-1.5.0.txt.asc

# Verify artifact integrity
sha256sum --check SHA256SUMS-1.5.0.txt --ignore-missing
```

This verifies both:
1. The checksums file is signed by Coincube (GPG signature)
2. The downloaded artifact matches the signed checksum (SHA256)

---

## Security Best Practices

- Use a dedicated release key only
- Never reuse personal GPG keys
- Limit passphrase knowledge
- Store backups encrypted and offline
- Rotate immediately on compromise

---

## Ownership

GPG key rotation requires approval from the CTO or Release Owner.

---

## Summary

This playbook ensures Coincube releases remain cryptographically trustworthy over time.
