# Security Policy

See [Engineering Standards §19](docs/engineering/ENGINEERING_STANDARDS.md#19-security-policy) for the full security policy.

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.x (`main`) | Active development (best-effort) |

## Reporting a Vulnerability

**Do not open public GitHub issues for security vulnerabilities.**

Email **security@synapticfour.com** with:

- Description of the issue
- Steps to reproduce
- Impact assessment if known

That address is the published contact for this repository. Acknowledgement targets (**48 hours** / triage **7 days**) are best-effort with one maintainer.

## Scope

In scope: S4MP CLI, workspace crates built by default, and CI for this repository.

Out of scope: parked crates (`s4-api`, `s4-ui`, `s4-planner`) until they ship a server; third-party code cloned via `s4 source add --git`.
