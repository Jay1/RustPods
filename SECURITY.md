# Security Policy

## Supported Versions

Currently, we provide security updates for the following versions of RustPods:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

The RustPods team takes security issues seriously. We appreciate your efforts to responsibly disclose your findings.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please follow these steps:

1. **Email**: Send details to [insert_email@example.com]
   - Replace this with your preferred security contact method

2. **Include Information**:
   - Type of issue (e.g., buffer overflow, SQL injection, etc.)
   - Full paths of source file(s) related to the vulnerability
   - Location of the affected source code (tag/branch/commit or direct URL)
   - Any special configuration required to reproduce the issue
   - Step-by-step instructions to reproduce the issue
   - Proof-of-concept or exploit code (if possible)
   - Impact of the issue, including how an attacker might exploit it

3. **Response Time**: We will acknowledge receipt of your vulnerability report within 48 hours and provide a detailed response within 5 business days.

4. **Disclosure**: The RustPods team will coordinate with you to determine an appropriate disclosure timeline once the issue is confirmed.

## Security Measures

RustPods implements the following security measures:

- **Bluetooth Data**: We only collect the minimum necessary data from your Bluetooth devices.
- **Local Storage**: All application data is stored locally on your device.
- **No Remote Services**: RustPods does not connect to any remote servers during normal operation.
- **Permissions**: We request only the Bluetooth permissions necessary for functionality.

## Security Considerations for Contributors

If you're contributing to RustPods, please keep these security best practices in mind:

1. **No Sensitive Data**: Never commit API keys, credentials, or personal information.
2. **Input Validation**: Validate all user input and Bluetooth data.
3. **Error Handling**: Implement proper error handling to prevent crashes and information disclosure.
4. **Dependencies**: Keep dependencies updated to address known vulnerabilities.

## Updates

We will revise this security policy as the project evolves. Check back for updates.

---

Last updated: May 2025 