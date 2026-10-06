# Security Policy

## Reporting Security Issues

**Do not** open public issues for security vulnerabilities. Instead, please email sibukixxx@gmail.com with:

- Description of the vulnerability
- Steps to reproduce (if applicable)
- Potential impact
- Suggested fix (if you have one)

We will acknowledge receipt within 48 hours and provide an update within 7 days.

## Security Considerations

### Input Validation

- Recipes are treated as untrusted input and validated for:
  - Path traversal (`..` in output paths)
  - Sensitive environment variable access (only `SF_*` and `DATABASE_URL` are allowed)
  - Table name escaping via `quote_ident()`

### Database Access

- SQLite sources are opened as read-only
- PostgreSQL connections enforce `SET TRANSACTION READ ONLY`
- TLS is enabled for remote connections (`tls-rustls`)

### Data Sensitivity

- Database connection strings are redacted in error messages and metadata
- Output paths are validated to prevent data exfiltration outside safe directories

## Supported Versions

Only the latest release version receives security updates. For critical issues, we may patch older versions on a case-by-case basis.

## Known Limitations

- This tool is not recommended for processing highly sensitive data without additional hardening
- Recipes should only come from trusted sources
- Deploy behind authentication in production environments
