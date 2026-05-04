# MSM P11 MCP Endpoint Design

## Scope

P11 adds the first MCP endpoint for MSM. It exposes the current pack
list/import/export capabilities through MCP tools over a JSON-RPC HTTP endpoint.

## References

- MCP tools require a declared `tools` server capability and are discovered with
  `tools/list`.
- Tools are invoked with `tools/call` and return content plus optional structured
  data.
- Streamable HTTP clients POST JSON-RPC messages to the MCP endpoint and servers
  may respond with one `application/json` JSON-RPC response.

Primary references:

- <https://modelcontextprotocol.io/specification/2025-06-18/server/tools>
- <https://modelcontextprotocol.io/specification/2025-06-18/schema>
- <https://modelcontextprotocol.io/specification/draft/basic/transports>

## Goals

- Add `msm-mcp` crate.
- Add `/mcp` route to `msm-app`.
- Support `initialize`, `ping`, `tools/list`, and `tools/call`.
- Expose current pack operations as MCP tools:
  - `msm.list_sticker_packs`
  - `msm.export_sticker_pack`
  - `msm.import_sticker_pack`
- Return deterministic tool schemas and structured content.
- Add focused route tests.

## Non-Goals

- No SSE stream support in P11.
- No MCP session management in P11.
- No auth/PAT enforcement in P11.
- No tools beyond the current API/CLI pack slice.

## Tool Contracts

`msm.list_sticker_packs`

- Input: `{ "userId": "user_1" }`
- Output: `{ "packs": [...] }`

`msm.export_sticker_pack`

- Input: `{ "packId": "pack_1" }`
- Output: `{ "pack": { ...MoreStickers sticker pack... } }`

`msm.import_sticker_pack`

- Input: `{ "tenantId": "...", "ownerUserId": "...", "packId": "...", "visibility": "public|private", "pack": {...} }`
- Output: `{ "imported": true, "packId": "..." }`

## Error Model

Unknown JSON-RPC methods use `-32601`. Invalid arguments use `-32602`. Unknown
tools use `-32602`. Storage and domain failures from tool execution are returned
as MCP tool results with `isError: true` where possible.
