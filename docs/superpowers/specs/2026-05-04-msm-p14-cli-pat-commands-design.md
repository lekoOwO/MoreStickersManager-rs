# MSM P14 CLI PAT Commands Design

## Scope

P14 exposes the P13 PAT management API through `msm-cli`. It adds create, list,
and revoke commands for Personal Access Tokens without changing API
authentication enforcement.

## Goals

- Add CLI commands:
  - `msm pats create`
  - `msm pats list`
  - `msm pats revoke`
- Add HTTP client methods for:
  - `POST /api/v1/pats`
  - `GET /api/v1/pats?userId=...`
  - `DELETE /api/v1/pats/{token_id}`
- Support repeated `--scope` arguments using stable permission keys such as
  `pack.read`, `asset.read`, and `pat.manage`.
- Support optional `--expires-at` as an RFC 3339 timestamp string passed through
  to the API.
- Print the raw token only for create responses.
- Keep JSON output available through the existing global `--output-format json`
  flag.
- Add CLI command parsing and execution tests.

## Non-Goals

- No PAT authentication enforcement on existing API, CLI, or MCP routes in P14.
- No local token persistence or config file.
- No Web UI PAT management.
- No scope discovery endpoint.

## CLI Contracts

Create:

```powershell
cargo run -p msm-cli -- pats create --id cli1 --user-id user_1 --name CLI --scope pack.read --scope asset.read
```

Human output:

```text
created cli1
msm_pat_cli1_secret
```

List:

```powershell
cargo run -p msm-cli -- pats list --user-id user_1
```

Human output:

```text
cli1	CLI	pack.read,asset.read
```

Revoke:

```powershell
cargo run -p msm-cli -- pats revoke --token-id cli1
```

Human output:

```text
revoked cli1
```
