# Security Policy

## Supported Versions

The following versions of LLM-TUI are currently supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of LLM-TUI seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to the project maintainer:

- **Email**: [Create a GitHub issue with the "security" label for private reporting](https://github.com/klt-mm/llm-tui/issues/new?template=security.md)
- **Response Time**: We aim to acknowledge receipt of vulnerability reports within 48 hours

### What to Include

Please provide as much information as possible to help us understand the nature and scope of the vulnerability:

1. **Description**: A clear description of the vulnerability
2. **Impact**: What an attacker could potentially achieve
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Proof of Concept**: Code or commands that demonstrate the vulnerability (if available)
5. **Affected Versions**: Which versions are affected
6. **Suggested Fix**: If you have suggestions for how to fix the issue

### What to Expect

After you submit a report:

1. **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
2. **Assessment**: We will assess the severity and impact of the vulnerability
3. **Investigation**: We will investigate and work on a fix
4. **Updates**: We will keep you informed of our progress
5. **Disclosure**: We will coordinate with you on the public disclosure timeline

### Disclosure Policy

- We follow responsible disclosure practices
- We will work with you to understand and validate the issue
- We will develop and test a fix
- We will release the fix and coordinate public disclosure
- We will credit reporters (unless they prefer to remain anonymous)

### Security Update Process

When a security vulnerability is identified:

1. A security advisory is created
2. A fix is developed and tested
3. The fix is released as a patch version
4. A security advisory is published with details
5. Users are notified via GitHub Security Advisories

## Security Best Practices

### For Users

1. **Keep Updated**: Always use the latest version of LLM-TUI
2. **API Keys**: Never commit API keys to version control
   - Use environment variables or configuration files
   - Add sensitive files to `.gitignore`
3. **Database Security**: 
   - The SQLite database contains your conversations
   - Ensure proper file permissions on `llm-tui.db`
   - Consider encrypting sensitive data at rest
4. **Network Security**:
   - Use HTTPS for API endpoints when possible
   - Verify SSL certificates
   - Be cautious when connecting to untrusted endpoints
5. **Tool Calling**:
   - Review tool calls before execution
   - Understand the permissions required by tools
   - Use in trusted environments only

### For Developers

1. **Input Validation**: Always validate and sanitize user input
2. **Dependencies**: Keep dependencies up to date
   - Run `cargo audit` regularly
   - Review dependency security advisories
3. **Error Handling**: Never expose sensitive information in error messages
4. **Logging**: Be cautious about what is logged
   - Never log API keys, tokens, or sensitive data
   - Use appropriate log levels
5. **Testing**: Write security-focused tests
   - Test edge cases
   - Test error conditions
   - Test with malformed input

## Security Considerations

### Data Storage

- Conversations are stored in a local SQLite database
- API keys are stored in configuration files
- No data is sent to external services except the configured LLM provider

### Network Communication

- All communication with LLM providers should use HTTPS
- API keys are sent in HTTP headers (Authorization: Bearer)
- No telemetry or analytics are collected

### Tool Execution

- Tools execute in the context of the user running LLM-TUI
- Tools have the same permissions as the user
- Exercise caution when using tool calling features
- Review tool calls before execution

### Third-Party Dependencies

LLM-TUI uses several third-party dependencies. We monitor these for security issues:

- Regular dependency audits with `cargo audit`
- Automated security checks via GitHub Actions
- Prompt updates when vulnerabilities are discovered

## Security Contacts

For security-related questions or concerns:

- **Primary**: [GitHub Security Advisory](https://github.com/klt-mm/llm-tui/security/advisories/new)
- **Secondary**: Open a private issue with the "security" label

## Security Updates

Security updates are released as patch versions (e.g., 1.0.1 -> 1.0.2).

To receive security updates:

1. Watch the repository on GitHub
2. Subscribe to releases
3. Follow security advisories

## Past Security Advisories

A list of past security advisories can be found in the [GitHub Security Advisories](https://github.com/klt-mm/llm-tui/security/advisories) section.

## Compliance

LLM-TUI is designed with security and privacy in mind:

- **Local-First**: Your data stays on your device
- **No Telemetry**: No data collection or analytics
- **Transparent**: Open source code for review
- **Minimal Dependencies**: Reduced attack surface

## Bug Bounty

Currently, we do not have a bug bounty program. However, we greatly appreciate responsible disclosure of security vulnerabilities.

## Acknowledgments

We would like to thank all security researchers who have responsibly disclosed vulnerabilities to help improve the security of LLM-TUI.

---

**Last Updated**: 2026-08-16

**Version**: 1.0
