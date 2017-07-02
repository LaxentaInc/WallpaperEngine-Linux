# Security Policy

## Supported Versions

Security updates are currently provided for the following versions of the Linux Wallpaper Engine. 

| Version | Supported          | Notes |
| ------- | ------------------ | ----- |
| 1.x.x   | :white_check_mark: | Active development and security patches |
| < 1.0   | :x:                | Deprecated, no longer receiving updates |

---

## Reporting a Vulnerability

Security is a top priority for us. If you discover a vulnerability within this project, please report it privately so we can safely patch it before public disclosure.

**Do not open a public GitHub issue for security vulnerabilities.**

### How to Report

Please report security issues via the **[GitHub Security Advisory](https://github.com/LaxentaInc/WallpaperEngine-Linux/security/advisories/new)** feature.

When submitting a report, please include as much information as possible:
* Type of vulnerability (e.g., XSS, buffer overflow, privilege escalation)
* Full details on how to reproduce the vulnerability
* Any environments (OS, Wayland compositor, X11, versions) where it is applicable
* Potential impact of the vulnerability

### Response Timeline

We take all security reports seriously and commit to the following timeline:
1. **Initial Response:** You will receive an acknowledgment of your report within **48 hours**.
2. **Triage:** We will confirm the vulnerability and determine its severity within **72 hours** of the initial response.
3. **Patching:** Critical vulnerabilities will receive a patch and advisory within **7 days**. Lower severity issues will be addressed in the next standard release cycle.

---

## Disclosure Policy

We follow a coordinated disclosure model. When a vulnerability is reported:
1. The issue is investigated and patched in a private fork.
2. A GitHub Security Advisory is drafted.
3. The patch is merged, a new release is cut, and the advisory is published simultaneously to ensure users can update immediately upon disclosure.

We ask that you refrain from publicly disclosing the vulnerability until the advisory and patch are released.

---

## Secure Architecture

This project is built with security in mind:
* **Rust:** We rely heavily on Rust's memory safety guarantees to prevent common vulnerabilities like buffer overflows and use-after-free bugs.
* **Tauri:** We utilize Tauri's strict IPC constraints and CSP (Content Security Policy) to isolate the webview frontend from the system-level backend.
* **Wayland:** By natively targeting the `wlr-layer-shell` protocol, we adhere to Wayland's strict window isolation policies, preventing keylogging or unauthorized screen capture from unprivileged clients.
