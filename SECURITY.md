# Security policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 1.2.x   | Yes       |
| <= 1.1  | No        |

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Email security concerns to the maintainers via GitHub private vulnerability reporting on [Pytxo-dev/pytxo](https://github.com/Pytxo-dev/pytxo/security/advisories/new), or contact the org owners directly.

We aim to acknowledge reports within 72 hours.

## Scope

- `pytxo` CLI, MCP server, and Rust crates
- Pytxo Desktop (`apps/desktop` in this monorepo)
- Pytxo Link, cloud sandbox, release installers, and the public web account boundary

Reports involving repository isolation, permission-profile enforcement,
review-package integrity, Apply/recovery evidence, credential exposure, or
receipt truthfulness are especially in scope. Include the affected version,
platform, reproduction steps, and whether any external side effect occurred.

Out of scope: vulnerabilities entirely inside third-party agent CLIs (Claude,
Codex, etc.) invoked via `pytxo run --cmd`. A Pytxo failure to contain, report,
or correctly attribute those effects remains in scope.
