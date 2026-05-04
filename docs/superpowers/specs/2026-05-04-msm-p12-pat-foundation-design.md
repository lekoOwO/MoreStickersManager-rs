# MSM P12 PAT Foundation Design

## Scope

P12 adds the storage and domain foundation for Personal Access Tokens. It does
not enforce PATs on API, CLI, or MCP routes yet.

## Goals

- Add stable permission scope keys in `msm-domain`.
- Add PAT token generation, hashing, verification, listing, and revocation in
  `msm-storage`.
- Store only token hashes in the database.
- Return the raw token only at creation time.
- Add tests for lifecycle behavior.

## Non-Goals

- No HTTP authentication middleware in P12.
- No Web UI PAT management in P12.
- No OIDC/SSO implementation in P12.
- No password/local account auth in P12.

## Token Format

PATs use:

```text
msm_pat_<token_id>_<random_secret>
```

The database stores `token_hash = sha256(random_secret)` and never stores the raw
token.

## Scope Format

Scopes use stable lowercase dot-separated permission keys, for example
`pack.read`, `asset.read`, and `pat.manage`. These map to
`msm_domain::Permission`.

## Verification

Verification parses the token, looks up the token ID, rejects revoked or expired
records, hashes the presented secret, and compares it to the stored hash.
