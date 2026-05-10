# MCP Transport Contract

Last updated: 2026-05-11.

## Scope

MSM's current MCP endpoint is intentionally stateless JSON-RPC over HTTP. The
contract is production-supported for request/response MCP clients and does not
claim stateful session or SSE-stream support.

## Endpoint

- URL: `POST /mcp`
- Content type: JSON-RPC 2.0 request body
- Supported methods: `initialize`, `ping`, `tools/list`, and `tools/call`
- Response caching: all MCP HTTP responses set `Cache-Control: no-store`

`GET /mcp` is not a negotiation endpoint. If a client asks for
`text/event-stream`, MSM returns a structured JSON error with
`mcp_sse_not_enabled`; other GET requests receive method-not-allowed JSON.

## Authentication

Discovery methods (`initialize`, `ping`, and `tools/list`) are public because
they expose protocol metadata and tool schemas only. Protected `tools/call`
operations must send the raw PAT in the HTTP header:

```http
Authorization: Bearer msm_pat_...
```

Clients must not send PATs inside JSON-RPC params. Tool handlers reuse the same
PAT verification, scope checks, tenant membership checks, and RBAC helpers used
by the API routes. Scope names are discoverable through `msm.get_pat_scope_policy`
when the caller has an appropriate management PAT.

## Session model

- MSM does not issue MCP session IDs.
- MSM does not keep per-client MCP server-side state.
- MSM does not support SSE resumability or stream reconnect state.
- Clients should treat every request as independent and retry only idempotent
  operations or explicit recovery tools.

If a future client requirement needs stateful MCP sessions or server-sent event
streams, that work must be added as a new PRD item with transport, lifecycle,
authentication, and replay tests before being considered supported.

## Proxy and deployment requirements

- Terminate TLS before exposing MCP outside localhost.
- Forward `POST /mcp` and the `Authorization` header unchanged.
- Do not cache MCP responses.
- Do not log `Authorization` headers or raw JSON-RPC request bodies containing
  sensitive arguments.
- Reject unexpected GET/SSE assumptions at the proxy only if it preserves the
  same client-visible semantics described above.

## Regression coverage

Current Rust regression tests cover the supported contract:

- `initialize_returns_capabilities`
- `mcp_post_disables_response_caching`
- `mcp_get_declares_sse_sessions_unsupported`
- `pat_enforcement_tools_call_requires_bearer`
- protected tool tests for pack, product metadata, tenant administration,
  provider import/config, export, publication history, and portability tools
