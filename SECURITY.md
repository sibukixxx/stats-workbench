# Security Policy

## Reporting a vulnerability

Do not open a public issue for vulnerabilities that may expose credentials, private endpoints, customer data, or exploitable details.

Use GitHub private vulnerability reporting for this repository when available. If private reporting is unavailable, contact the maintainer privately through the contact method listed on the maintainer's GitHub profile.

## Security considerations

- Recipes and external data sources are treated as untrusted input.
- Output paths are validated against path traversal.
- Only explicitly allowed environment variables may be expanded from recipes.
- SQLite sources are opened read-only.
- PostgreSQL analysis uses read-only transaction semantics where applicable.
- Database connection strings and secrets must not be emitted in logs or artifacts.
- Do not commit credentials, private datasets, or production database snapshots.

## Supported code

Security fixes target the current `main` branch until stable releases are published.
