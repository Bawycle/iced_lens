# Security Policy

## Supported Versions

As IcedLens is currently in pre-1.0 development, security updates are provided for the latest release only.

| Version | Supported          |
| ------- | ------------------ |
| 0.6.x   | :white_check_mark: |
| 0.5.x   | :x:                |
| < 0.5   | :x:                |

## Security Context

IcedLens is a local-first image viewer and editor:
- **Local processing**: Images and videos are processed entirely on your machine
- **Minimal network activity**: The only network connections are optional AI features, which download ONNX models from Hugging Face on first use: NAFNet for deblurring (~92 MB) and Real-ESRGAN for upscaling (~64 MB). Each model's integrity is verified with a BLAKE3 checksum.
- **No telemetry**: No usage data, analytics, or tracking sent to any server
- **Privacy-safe diagnostics**: The optional diagnostics feature (Settings → Diagnostics) collects anonymized events locally for troubleshooting. All file paths are hashed with blake3 (8-character hash). Data stays on your device unless you explicitly export and share it.

Security vulnerabilities may arise from:
- Malformed image files triggering bugs in image decoding libraries
- Path traversal or file access issues
- Unsafe handling of user inputs (zoom values, file paths, etc.)
- Vulnerabilities in dependencies

## Reporting a Vulnerability

If you discover a security vulnerability in IcedLens, please help us address it responsibly.

### How to Report

**Please DO NOT open a public issue for security vulnerabilities.**

Instead, report security issues privately through one of these methods:

1. **Preferred: Private issue on Codeberg**
   - Go to the [IcedLens repository](https://codeberg.org/Bawycle/iced_lens)
   - Open a new issue and mark it as **confidential** (if available)
   - Provide detailed information about the vulnerability (see below)

2. **Alternative: Email**
   - Contact the maintainer directly (contact information available in Git commits or project documentation)
   - Use the subject line: "IcedLens Security Vulnerability Report"

### What to Include in Your Report

To help us address the vulnerability quickly, please include:

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Affected versions** (e.g., "0.1.0 and possibly earlier")
- **Potential impact** (what could an attacker accomplish?)
- **Suggested fix** (if you have one)
- **Sample files** (if the issue is triggered by a specific image file)

### What to Expect

After you submit a vulnerability report:

1. **Acknowledgment**: We'll acknowledge receipt within **72 hours**
2. **Assessment**: We'll assess the severity and confirm the vulnerability
3. **Fix Development**: We'll work on a fix and keep you updated on progress
4. **Disclosure Timeline**: We aim to release a fix within **30 days** for critical issues
5. **Credit**: You'll be credited in the release notes (unless you prefer to remain anonymous)

### Disclosure Policy

- We follow **responsible disclosure** principles
- Security fixes will be released as soon as possible
- A security advisory will be published after the fix is released
- We'll coordinate with you on the disclosure timeline if you're the reporter

## Security Best Practices for Users

While using IcedLens:

- **Keep your installation up to date** with the latest release
- **Be cautious** when opening image files from untrusted sources
- **Run `cargo audit`** if you're building from source to check for vulnerable dependencies
- **Report suspicious behavior** even if you're not sure it's a security issue

## Dependency Security

IcedLens relies on several third-party libraries for image processing. We:

- Regularly run `cargo audit` to check for known vulnerabilities in dependencies
- Update dependencies when security patches are available
- Monitor security advisories for the Rust ecosystem

If you discover a vulnerability in one of IcedLens's dependencies:
- Report it to the upstream project first
- Notify us so we can update the dependency version

## Security-Related Development Practices

Contributors should follow these security practices:

- **Validate all user inputs** (file paths, zoom values, crop dimensions, etc.)
- **Avoid `unwrap()` on user-provided data** (use proper error handling)
- **Use safe Rust** (avoid `unsafe` blocks unless absolutely necessary and well-justified)
- **Run `cargo clippy`** to catch potential issues
- **Test with malformed inputs** (fuzzing-style testing when applicable)

See [CONTRIBUTING.md](CONTRIBUTING.md) for full development guidelines.

---

Thank you for helping keep IcedLens and its users safe!
