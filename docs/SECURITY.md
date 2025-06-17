# RustPods Security Policy: Technical Standards and Protocols

## Supported Versions

The RustPods project provides security maintenance for the following release series:

| Version | Maintenance Status    |
| ------- | -------------------- |
| 0.1.x   | Supported            |

## Vulnerability Reporting Protocol

The RustPods engineering team maintains a rigorous approach to security incident management. Responsible disclosure is essential to the integrity of the project.

**Do not disclose security vulnerabilities via public GitHub issues.**

### Reporting Procedure

1. **Confidential Submission**: Communicate vulnerability details directly to the security team at [j-chiasson@outlook.com].
2. **Required Information**:
   - Classification of the vulnerability (e.g., buffer overflow, injection, etc.)
   - Full source file paths implicated in the issue
   - Precise code location (tag/branch/commit or direct URL)
   - Any special configuration required for reproduction
   - Stepwise reproduction instructions
   - Proof-of-concept or exploit code, if available
   - Assessment of impact and potential exploitation vectors
3. **Response Commitment**: The team will acknowledge receipt within 48 hours and provide a comprehensive response within 5 business days.
4. **Coordinated Disclosure**: Disclosure timelines will be determined in collaboration with the reporter upon confirmation of the issue.

## Security Architecture and Controls

- **Bluetooth Data Minimization**: Only essential Bluetooth data is collected.
- **Local Data Residency**: All operational data is stored exclusively on the user's device.
- **No Remote Connectivity**: RustPods does not initiate outbound connections to remote servers during standard operation.
- **Principle of Least Privilege**: Only the minimum necessary Bluetooth permissions are requested.

## Contributor Security Standards

Contributors are required to adhere to the following security engineering practices:

1. **No Sensitive Data in Source Control**: Never commit API keys, credentials, or personal data.
2. **Input Validation**: Rigorously validate all user and Bluetooth input.
3. **Robust Error Handling**: Implement comprehensive error handling to prevent system crashes and information leakage.
4. **Dependency Management**: Maintain up-to-date dependencies to mitigate known vulnerabilities.

## Policy Maintenance

This security policy is subject to revision as the project evolves. Contributors and users are advised to consult this document regularly for updates.

---

_Last revised: May 2025_ 