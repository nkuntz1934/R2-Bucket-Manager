# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| latest  | ✅                |
| < latest| ❌                |

## Reporting a Vulnerability

We take the security of Rust R2 seriously. If you have discovered a security vulnerability, please follow these steps:

### 1. Do NOT Create a Public Issue

Security vulnerabilities should not be reported through public GitHub issues.

### 2. Email the Maintainers

Send details to the project maintainers through:
- GitHub Security Advisories (preferred)
- Direct email to maintainers (check commit history for emails)

### 3. Include Details

Please include:
- Type of vulnerability
- Full paths of affected source files
- Location of affected code (tag/branch/commit or direct URL)
- Step-by-step instructions to reproduce
- Proof-of-concept or exploit code (if possible)
- Impact of the issue

### 4. Response Time

- Initial response: Within 48 hours
- Status update: Within 5 business days
- Resolution timeline: Depends on severity

## Security Best Practices

### For Users

#### Credentials Management
- **Never commit credentials** to version control
- Use environment variables or secure vaults
- Rotate API tokens regularly
- Use minimal required permissions

#### PGP Keys
- Generate strong keys (minimum 2048-bit RSA)
- Use passphrases for private keys
- Store keys separately from data
- Backup keys securely
- Never share private keys

#### Configuration Files
- Set restrictive permissions:
  ```bash
  chmod 600 config.json
  chmod 600 *.key
  ```
- Use `.gitignore` for sensitive files
- Encrypt configuration in transit
- Audit configuration regularly

#### Network Security
- Use HTTPS endpoints only
- Verify SSL certificates
- Use VPN for sensitive operations
- Monitor network traffic

### For Developers

#### Code Security
- Validate all inputs
- Sanitize file paths
- Use safe Rust patterns
- Avoid unsafe code blocks
- Handle errors explicitly

#### Dependencies
- Keep dependencies updated
- Audit dependencies regularly
- Use `cargo audit` tool
- Pin critical dependencies
- Review dependency changes

#### Encryption
- Use established libraries
- Don't implement custom crypto
- Use secure random generators
- Validate encryption parameters
- Test encryption/decryption

## Known Security Considerations

### File Operations
- Files are loaded into memory (not streamed)
- Large files may cause memory issues
- Temporary files may contain sensitive data

### PGP Implementation
- Uses `pgp` crate for encryption
- Supports RSA and ECC keys
- No signature verification currently
- Passphrase stored in memory during session

### Network Communication
- Direct HTTPS to Cloudflare R2
- No proxy support currently
- No certificate pinning
- Standard TLS validation

### Authentication
- API keys stored in configuration
- No token refresh mechanism
- No MFA support
- Credentials in environment variables

## Security Checklist

### Before Deployment
- [ ] Remove all debug code
- [ ] Disable verbose logging
- [ ] Secure configuration files
- [ ] Set up access controls
- [ ] Review firewall rules
- [ ] Enable audit logging

### Regular Maintenance
- [ ] Update dependencies monthly
- [ ] Rotate credentials quarterly
- [ ] Review access logs
- [ ] Test backup/recovery
- [ ] Audit configuration
- [ ] Security scanning

## Vulnerability Disclosure

### Timeline
1. **Day 0**: Vulnerability reported
2. **Day 1-2**: Initial assessment
3. **Day 3-7**: Develop fix
4. **Day 8-14**: Test and review
5. **Day 15**: Release patch
6. **Day 30**: Public disclosure

### Credit
We credit security researchers who:
- Report vulnerabilities responsibly
- Allow time for patching
- Don't exploit vulnerabilities
- Help test fixes

## Security Updates

### Notification
Security updates are announced through:
- GitHub Security Advisories
- Release notes
- README updates

### Update Process
1. Check for updates regularly
2. Review security advisories
3. Test in non-production first
4. Apply updates promptly
5. Verify functionality

## Compliance

This project aims to follow security best practices including:
- OWASP guidelines
- Rust security recommendations
- Cloudflare security requirements
- OpenPGP standards

## Contact

For security concerns, contact:
- GitHub Security Advisory (preferred)
- Project maintainers
- Open an issue (for non-sensitive concerns)

## Acknowledgments

We thank the security researchers and users who help keep Rust R2 secure through responsible disclosure.

## Resources

- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Cloudflare Security](https://www.cloudflare.com/security/)
- [OpenPGP Best Practices](https://riseup.net/en/security/message-security/openpgp/best-practices)