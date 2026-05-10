# Security Review: Tokens, Secret Redaction, and Asset Access

Date: 2026-05-11
Scope: Phase K production-hardening review of credential storage, response redaction, and private asset/subscription access paths.

This document records the current security posture and residual risks. It is not a substitute for an external penetration test before high-risk deployments.

## Summary

| Area | Status | Evidence | Residual risk / follow-up |
| --- | --- | --- | --- |
| PAT storage | Acceptable for current self-hosted scope | Raw PATs are returned once from storage creation; only `token_hash` is persisted and verification compares the presented secret hash with constant-time equality. API list responses omit raw token secrets/hashes. | SHA-256 without a server-side pepper means database disclosure enables offline guessing if token entropy is ever weakened. Keep generated secrets high entropy; consider HMAC/pepper in a future security-hardening phase. |
| Web session storage | Acceptable | `msm_session` raw value is returned once in an HttpOnly/SameSite=Lax cookie; only `session_hash` is stored; verification rejects revoked/expired sessions. | Cookie currently lacks a runtime Secure flag toggle documented for HTTPS deployments; add Secure cookie configuration before public multi-user production. |
| Subscription access-token storage | Acceptable | `msm_sub_*` secrets are returned only on create/rotate; only `token_hash` is stored; list responses are metadata-only. | Same SHA-256/pepper consideration as PATs. Operators must treat subscription links as bearer credentials. |
| OIDC login state/nonce | Acceptable | State and nonce raw secrets are returned once; only hashes are stored; callback verifies state, nonce, expiration, and consumes state. | Continue keeping callback logs free of state/nonce query values. Structured HTTP logs currently use path only, not query string. |
| Local password storage | Acceptable | Local credentials use Argon2 PHC hashes with per-password salt via `argon2`. | Password policy/rate limiting for login attempts is still a future hardening item. |
| Export target/provider config redaction | Acceptable | API and MCP recursively redact JSON keys containing `token` or `secret`; OIDC provider responses return `[redacted]` client secret. Tests assert real Telegram/provider/OIDC secrets are absent from responses. | Heuristic redaction will miss non-standard names such as `password`, `privateKey`, or opaque nested blobs. Extend the redaction key list before accepting arbitrary third-party provider schemas. |
| Prepared media / job diagnostics | Needs operator care | Converter stdout/stderr are surfaced for diagnostics. | External tools could emit sensitive paths or provider URLs. Operators should avoid embedding secrets in media paths/arguments; future work can add configurable diagnostics truncation/redaction. |
| Private asset access | Acceptable for current model | Private asset reads require owner `asset.read` PAT, owner Web session, matching pack subscription token, or subscription-group token containing the pack. Anonymous reads of private pack assets are rejected. | Subscription tokens are bearer credentials and should be rotated if leaked. Asset backing store is local filesystem; enforce OS-level permissions around `MSM_ASSET_DIR`. |
| Public pack/subscription access | Acceptable | Public pack/subscription payloads remain anonymous; private packs are filtered from anonymous public subscription-group reads. | Operators must understand that setting pack visibility public intentionally publishes asset URLs. |
| CDN/public asset URLs | Acceptable with operator controls | Tenant `publicAssetUrl` and system `MSM_PUBLIC_ASSET_URL` only rewrite served payload URLs; tenant setting takes precedence. | Misconfigured CDN origins can expose private assets if the CDN bypasses MSM authorization. Private asset paths should not be cached publicly unless protected by the same credential model. |
| MCP transport | Partially hardened | `/mcp` is stateless JSON-RPC over POST, sets no-store, and rejects SSE GET negotiation with structured unsupported-session response. Tool calls use Bearer PAT enforcement. | PRD still tracks final MCP session/auth/SSE contract closure because stateful MCP sessions are intentionally not implemented. |

## Detailed findings

### 1. Token and session storage

Current generated bearer credentials use a stable prefix and random secret segment:

- PAT: `msm_pat_<token_id>_<random_secret>`
- Web session: `msm_session_<session_id>_<random_secret>`
- Subscription access token: `msm_sub_<token_id>_<random_secret>`
- OIDC state/nonce: `msm_oidc_state_*` / `msm_oidc_nonce_*`

Storage persists hashes (`token_hash`, `session_hash`, `state_hash`, `nonce_hash`) rather than raw secrets. Verification parses the presented token, hashes only the secret portion, and compares using constant-time equality where implemented in repository verification paths.

Operational requirements:

- Treat all raw `msm_pat_*`, `msm_session_*`, and `msm_sub_*` values as bearer secrets.
- Never store raw PAT/subscription/session values in issue trackers, screenshots, or logs.
- Rotate PATs/subscription links after suspected disclosure.
- Keep database backups encrypted because hashes and OIDC/client/provider secrets remain sensitive.

### 2. Secret redaction review

Current redaction surfaces:

- Export target config API/MCP responses recursively redact keys containing `token` or `secret`.
- Provider config API/MCP responses use the same recursive token/secret redaction.
- OIDC provider API/MCP/Web surfaces expose client secret as `[redacted]` rather than raw value.
- PAT/subscription-link list endpoints return metadata only; raw secrets are only returned on create/rotate.

Known limitation: redaction is key-name heuristic based. Before adding new provider/exporter schemas, review whether secrets use names outside `token`/`secret` and add tests for those names.

### 3. Asset access review

Private MSM-hosted assets are protected at the API path level. The accepted credentials are intentionally narrow:

- same owner PAT with `asset.read` and tenant membership;
- same owner Web session cookie;
- matching pack subscription token;
- subscription-group token that includes the requested pack.

Anonymous requests can read assets whose pack record is public. Asset paths without a pack record remain readable for backward compatibility; operators should avoid leaving orphaned sensitive files under `MSM_ASSET_DIR`.

CDN guidance:

- Do not configure a CDN rule that fetches private assets directly from disk/object storage while bypassing MSM.
- Prefer CDN rules that forward Authorization/Cookie headers and respect MSM `401/403` responses for private asset paths.
- Purge CDN cache after changing a pack from public to private.

### 4. Logging review

Structured request logs record method, path, status, and elapsed milliseconds. They intentionally omit query strings, headers, cookies, and request bodies. This avoids logging PATs, OIDC state/code query strings, subscription tokens, and provider credentials.

Operator requirement: keep reverse proxy/access logs aligned with this policy. If a proxy logs full query strings or Authorization headers, redact or disable those fields.

### 5. Recommended follow-up backlog

These items are not blockers for the current Phase K review checkbox, but should be considered before high-risk hosted deployments:

1. Add configurable Secure cookie flag / trusted-proxy HTTPS setting for `msm_session`.
2. Replace SHA-256 token hashes with HMAC-SHA-256 using a server-side secret pepper, or Argon2id for low-volume bearer tokens.
3. Extend redaction keys to include `password`, `private_key`, `privateKey`, `apiKey`, `accessKey`, and provider-specific aliases.
4. Add login-attempt rate limiting and audit events for failed auth attempts.
5. Add configurable truncation/redaction for converter stdout/stderr diagnostics.
6. Add orphaned asset scan/report tooling for files not referenced by pack records.

## Review decision

The current implementation satisfies the PRD requirement for a security review of token storage, secret redaction, and asset access. The implementation is acceptable for self-hosted deployments that follow the documented backup, CDN, and credential-handling guidance. The residual risks above remain tracked as future hardening candidates rather than open blockers for this PRD review item.
