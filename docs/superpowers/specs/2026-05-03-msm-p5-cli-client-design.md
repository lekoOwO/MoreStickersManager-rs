# MSM P5 CLI Client Design

Date: 2026-05-03
Phase: P5

## Purpose

P5 adds a scriptable CLI client for MSM. The CLI should be usable by humans and automation, share request/response shapes with the API where practical, and support the first P4 HTTP slice: health checks, pack import, pack listing, and pack export.

## Scope

In scope for the first P5 slice:
- `msm-cli` crate with an `msm` binary.
- Clap-based command parser.
- HTTP client abstraction.
- Reqwest-backed API client.
- JSON and human output modes.
- Commands:
  - `msm health`
  - `msm packs list --user-id <id>`
  - `msm packs import --tenant-id <id> --owner-user-id <id> --pack-id <id> --visibility <public|private> --file <path>`
  - `msm packs export --pack-id <id> --output <path|->`
- Unit tests for command execution through a fake client.
- CLI docs and agent testing notes.

Out of scope:
- Login/session management.
- PAT persistence.
- MCP-specific flows.
- Provider import jobs.
- Direct embedded server mode.
- Shell completions.

## Architecture

```text
crates/
  msm-cli/
    src/
      client.rs
      command.rs
      error.rs
      lib.rs
      main.rs
      output.rs
```

The binary is thin:

```rust
#[tokio::main]
async fn main() {
    msm_cli::run_from_env().await
}
```

Most behavior lives in library functions so tests do not spawn subprocesses.

## Command Model

Global flags:
- `--base-url <url>` default: `http://127.0.0.1:8080`
- `--output-format <human|json>` default: `human`

Commands:

```text
msm health
msm packs list --user-id user_1
msm packs import --tenant-id tenant_1 --owner-user-id user_1 --pack-id pack_1 --visibility private --file pack.stickerpack
msm packs export --pack-id pack_1 --output pack.stickerpack
msm packs export --pack-id pack_1 --output -
```

## HTTP Client Contract

`MsmClient` trait:
- `health() -> HealthResponse`
- `list_packs(user_id) -> Vec<StickerPack>`
- `import_pack(request) -> ()`
- `export_pack(pack_id) -> StickerPack`

`ReqwestMsmClient` maps to P4 endpoints:
- `GET /healthz`
- `GET /api/v1/packs?userId=...`
- `POST /api/v1/packs/import`
- `GET /api/v1/packs/{pack_id}/stickerpack`

The CLI does not yet handle auth headers. P10/P5 follow-up will add `--token`, env vars, and config file support.

## Output

Human output:
- health: `ok`
- list: one line per pack: `<id>\t<title>`
- import: `imported <pack_id>`
- export to file: `exported <pack_id> to <path>`
- export to stdout: raw sticker pack JSON

JSON output:
- health: API health JSON
- list: sticker pack JSON array
- import: `{"status":"imported","packId":"..."}`
- export: sticker pack JSON

## Testing

Unit tests use a fake `MsmClient`:
- `health` prints `ok`;
- `packs list` prints pack ID/title;
- `packs import` reads a fixture file and calls fake import;
- `packs export --output -` prints JSON;
- invalid input returns a typed CLI error.

HTTP integration tests are deferred until a service binary exists.

## Documentation Updates

Update:
- `README.md` with CLI command examples.
- `docs/user/README.md` with current CLI status.
- `docs/agents/project-map.md` with `msm-cli`.
- `docs/agents/testing.md` with `cargo test -p msm-cli`.
- `docs/status/current.md` and checkpoints.

## Design Decisions

1. CLI is an HTTP client, not a direct storage manipulator.
2. Tests use a fake client so CLI command behavior remains deterministic.
3. `--output -` means stdout for export.
4. Auth is intentionally deferred until PAT/session work exists.
