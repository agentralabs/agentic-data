# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in AgenticData, please report it responsibly:

1. **Do NOT** open a public GitHub issue.
2. Email: security@agentralabs.tech
3. Include: description, reproduction steps, impact assessment.
4. You will receive acknowledgment within 48 hours.
5. We aim to release a fix within 7 days of confirmed vulnerabilities.

## Security Considerations

- **Field-level encryption** uses BLAKE3-derived per-field keys. Production deployments should use ChaCha20-Poly1305 via the `ring` crate.
- **PII redaction** is pattern-based. Context-dependent PII (name + address combinations) requires LLM-assisted detection for full coverage.
- **MCP server** auth: Server profile requires `AGENTIC_TOKEN` environment variable.
- **.adat files** store connection strings encrypted. Master keys are never stored in the file.
