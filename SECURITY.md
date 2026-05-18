# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| 0.1.x   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

1. **Do not** open a public issue
2. Email the maintainer directly or use GitHub's private vulnerability reporting
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- Acknowledgment within 48 hours
- Initial assessment within 1 week
- Fix timeline depends on severity
- Credit in security advisory (optional)

## Security Best Practices

When using wzllama:

1. **API Keys**: Never commit API keys. Use environment variables
2. **Model Downloads**: Models are downloaded from trusted sources (HuggingFace, Ollama)
3. **Local Data**: All data stays local unless you configure external services
4. **Updates**: Keep wzllama updated for security fixes

## Known Security Considerations

- Model weights are downloaded from HuggingFace and Ollama registries
- Some tools may require elevated privileges (sudo)
- Generated code should be reviewed before execution

Thank you for helping keep wzllama secure!