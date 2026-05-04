# MSM P14 CLI PAT Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add CLI create/list/revoke commands for Personal Access Tokens.

**Architecture:** Keep `msm-cli` as a thin HTTP client. Command parsing lives in `command.rs`, wire DTOs and HTTP calls in `client.rs`, and keep human/JSON formatting in `output.rs`.

**Tech Stack:** Rust, clap, async-trait, reqwest, serde.

---

### Task 1: CLI PAT Planning Checkpoint

**Files:**
- Create: `docs/superpowers/specs/2026-05-04-msm-p14-cli-pat-commands-design.md`
- Create: `docs/superpowers/plans/2026-05-04-msm-p14-cli-pat-commands.md`

- [ ] **Step 1: Commit planning docs**

```powershell
git add docs/superpowers/specs/2026-05-04-msm-p14-cli-pat-commands-design.md docs/superpowers/plans/2026-05-04-msm-p14-cli-pat-commands.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: add P14 CLI PAT commands plan"
```

### Task 2: Add Failing CLI PAT Tests

**Files:**
- Modify: `crates/msm-cli/src/command.rs`
- Modify: `crates/msm-cli/src/client.rs`
- Modify: `crates/msm-cli/src/output.rs`

- [ ] **Step 1: Add parse tests**

Add tests named `parses_pat_create_command`, `parses_pat_list_command`, and
`parses_pat_revoke_command` in `crates/msm-cli/src/command.rs`.

- [ ] **Step 2: Add execution tests**

Add tests named `executes_pat_create_command`, `executes_pat_list_command`, and
`executes_pat_revoke_command` in `crates/msm-cli/src/command.rs`.

- [ ] **Step 3: Verify tests fail before implementation**

```powershell
cargo test -p msm-cli pats
```

Expected: compile failure because `Command::Pats`, `PatCommand`, PAT DTOs, and
client trait methods do not exist yet.

### Task 3: Implement CLI PAT Commands

**Files:**
- Modify: `crates/msm-cli/src/command.rs`
- Modify: `crates/msm-cli/src/client.rs`
- Modify: `crates/msm-cli/src/output.rs`

- [ ] **Step 1: Add command models**

Add `Command::Pats { command: PatCommand }` and `PatCommand::{Create, List,
Revoke}`. `Create` accepts `--id`, `--user-id`, `--name`, repeated `--scope`,
and optional `--expires-at`.

- [ ] **Step 2: Add client DTOs and trait methods**

Add `CreatePersonalAccessTokenPayload`, `CreatedPersonalAccessToken`,
`PersonalAccessToken`, and methods `create_pat`, `list_pats`, and `revoke_pat`
to `MsmClient`.

- [ ] **Step 3: Add reqwest calls**

Implement:

```text
POST /api/v1/pats
GET /api/v1/pats?userId=<user_id>
DELETE /api/v1/pats/<token_id>
```

- [ ] **Step 4: Add output formatters**

Human create output must be two lines: `created <id>` and the raw token. Human
list output must be tab-separated `id`, `name`, and comma-separated scopes.
Human revoke output must be `revoked <token_id>`. JSON output must use pretty
serde JSON.

- [ ] **Step 5: Verify focused CLI checks**

```powershell
cargo fmt --all -- --check
cargo test -p msm-cli pats
cargo clippy -p msm-cli --all-targets -- -D warnings
```

- [ ] **Step 6: Commit implementation**

```powershell
git add crates/msm-cli
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "feat: add CLI PAT commands"
```

### Task 4: Documentation And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `docs/user/README.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Document PAT CLI usage**

Document create/list/revoke examples and keep the non-enforcement note explicit.

- [ ] **Step 2: Run full verification**

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run web:typecheck
npm run web:test
npm run web:build
```

- [ ] **Step 3: Commit documentation**

```powershell
git add README.md docs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit -m "docs: update CLI PAT command status"
```
