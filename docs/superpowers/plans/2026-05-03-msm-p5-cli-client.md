# MSM P5 CLI Client Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first scriptable MSM CLI client for the P4 API slice.

**Architecture:** Create `msm-cli` as a library-backed binary. Keep command execution generic over an `MsmClient` trait so tests use fake clients without running an HTTP server.

**Tech Stack:** Rust, clap, reqwest, async-trait, serde_json, tokio.

---

## Task 1: Scaffold CLI Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/msm-cli/Cargo.toml`
- Create: `crates/msm-cli/src/lib.rs`
- Create: `crates/msm-cli/src/main.rs`
- Create: `crates/msm-cli/src/error.rs`

- [ ] Add `crates/msm-cli` to workspace.
- [ ] Add dependencies: `async-trait`, `clap`, `msm-domain`, `reqwest`, `serde`, `serde_json`, `thiserror`, `tokio`, `url`.
- [ ] Add `CliError`, `CliResult`, and thin binary entrypoint.
- [ ] Run `cargo test -p msm-cli`.
- [ ] Commit with `chore: scaffold CLI crate`.

## Task 2: Command Model and Output

**Files:**
- Create: `crates/msm-cli/src/command.rs`
- Create: `crates/msm-cli/src/output.rs`
- Modify: `crates/msm-cli/src/lib.rs`

- [ ] Define clap parser, global flags, commands, visibility enum, and output format enum.
- [ ] Implement output helpers for human/json responses.
- [ ] Add parser tests for all target commands.
- [ ] Run `cargo test -p msm-cli command`.
- [ ] Commit with `feat: add CLI command model`.

## Task 3: Client Trait and Reqwest Client

**Files:**
- Create: `crates/msm-cli/src/client.rs`
- Modify: `crates/msm-cli/src/lib.rs`

- [ ] Define `MsmClient` trait.
- [ ] Implement `ReqwestMsmClient`.
- [ ] Map methods to P4 endpoints.
- [ ] Add URL joining tests.
- [ ] Run `cargo test -p msm-cli client`.
- [ ] Commit with `feat: add CLI HTTP client`.

## Task 4: Command Execution

**Files:**
- Modify: `crates/msm-cli/src/lib.rs`
- Modify: `crates/msm-cli/src/command.rs`
- Modify: `crates/msm-cli/src/output.rs`

- [ ] Implement `execute_with_client`.
- [ ] Implement `run_from_env`.
- [ ] Add fake-client tests for health, list, import, export stdout, and invalid file.
- [ ] Run `cargo test -p msm-cli`.
- [ ] Commit with `feat: add CLI command execution`.

## Task 5: Docs and Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] Document CLI examples.
- [ ] Update status and checkpoints.
- [ ] Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] Commit with `docs: update P5 CLI status`.

## Completion Criteria

- `msm-cli` command tests pass.
- Workspace format, clippy, and tests pass.
- CLI does not bypass HTTP API behavior.
