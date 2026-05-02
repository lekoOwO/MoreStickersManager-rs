# MSM P0/P1 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the MSM project foundation and MoreStickers-compatible domain core.

**Architecture:** Start with repository hygiene, documentation, and a minimal Rust workspace. Keep compatibility models in `crates/msm-domain` with no API, database, provider SDK, or frontend dependencies.

**Tech Stack:** Rust workspace, serde, serde_json, thiserror, url, cargo test, GitHub Actions, Markdown docs.

---

## Scope

This plan implements only P0 and P1 from `docs/superpowers/specs/2026-05-02-msm-platform-roadmap-foundation-design.md`.

In scope:
- repository hygiene;
- status and agent handoff docs;
- Rust workspace scaffold;
- `msm-domain` compatibility structs;
- provider ID helper functions;
- asset URL resolver;
- `.stickerpack` import/export helpers;
- golden fixtures and tests;
- CI baseline.

Out of scope:
- Axum API;
- SQLx storage;
- Web UI;
- MCP endpoint;
- provider network fetches;
- OIDC, PATs, RBAC implementation.

## Current Repo Assumptions

The repo currently has one committed spec and several untracked local files:
- `.agents/`
- `.claude/`
- `node_modules/`
- `package-lock.json`
- `package.json`
- `skills-lock.json`

Do not include these untracked local files in implementation commits unless a task explicitly names them.

## Target File Structure

```text
.github/
  workflows/
    ci.yml
.gitignore
Cargo.toml
README.md
crates/
  msm-domain/
    Cargo.toml
    src/
      error.rs
      ids.rs
      lib.rs
      stickerpack.rs
      url.rs
    tests/
      compatibility.rs
      fixtures/
        dynamic_pack_set.json
        line_emoji_pack.stickerpack.json
        line_sticker_pack.stickerpack.json
        telegram_pack.stickerpack.json
docs/
  agents/
    README.md
    compatibility.md
    project-map.md
    status-protocol.md
    testing.md
  dev/
    architecture.md
    compatibility.md
  status/
    checkpoints.md
    current.md
    decisions.md
  superpowers/
    plans/
      2026-05-02-msm-p0-p1-foundation.md
    specs/
      2026-05-02-msm-platform-roadmap-foundation-design.md
  user/
    README.md
```

## Task 1: Repository Hygiene

**Files:**
- Create: `.gitignore`
- Modify: `docs/status/current.md` if it already exists from a parallel worker

- [ ] **Step 1: Write `.gitignore`**

Create `.gitignore` with this content:

```gitignore
# Rust
/target/
**/*.rs.bk

# Node and frontend builds
/node_modules/
**/node_modules/
/dist/
**/dist/
/.vite/
**/.vite/

# Environment and secrets
.env
.env.*
!.env.example
!.env.*.example

# Local data
/data/
*.sqlite
*.sqlite3
*.db
*.db-wal
*.db-shm

# Logs and coverage
*.log
/coverage/
**/coverage/
lcov.info

# OS and editor noise
.DS_Store
Thumbs.db
.idea/
.vscode/
*.swp
*.swo

# Temporary files
/tmp/
*.tmp
*.bak
```

- [ ] **Step 2: Verify ignored local dependencies**

Run:

```powershell
git status --short
```

Expected:

```text
?? .gitignore
?? docs/
```

Acceptable additional untracked files:

```text
?? .agents/
?? .claude/
?? package-lock.json
?? package.json
?? skills-lock.json
```

`node_modules/` must not appear after `.gitignore` is created.

- [ ] **Step 3: Commit repository hygiene**

Run:

```powershell
git add .gitignore
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "chore: add repository ignore rules"
```

Expected:

```text
[main <hash>] chore: add repository ignore rules
```

## Task 2: Human and Agent Documentation Baseline

**Files:**
- Create: `README.md`
- Create: `docs/dev/architecture.md`
- Create: `docs/dev/compatibility.md`
- Create: `docs/user/README.md`
- Create: `docs/agents/README.md`
- Create: `docs/agents/project-map.md`
- Create: `docs/agents/compatibility.md`
- Create: `docs/agents/status-protocol.md`
- Create: `docs/agents/testing.md`
- Create: `docs/status/current.md`
- Create: `docs/status/decisions.md`
- Create: `docs/status/checkpoints.md`

- [ ] **Step 1: Write `README.md`**

Create `README.md`:

```markdown
# MoreStickersManager-rs

MoreStickersManager-rs, abbreviated MSM, is a Rust self-hosted manager for MoreStickers-compatible sticker packs.

Current phase: P0/P1 foundation and format compatibility.

## Compatibility Target

MSM preserves the `.stickerpack` JSON shape used by Equicord moreStickers and MoreStickersConverter. The compatibility source of truth is documented in `docs/dev/compatibility.md`.

## Development

Run the current baseline checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Before the Rust workspace exists, use:

```powershell
git status --short
```

## Project Docs

- `docs/dev/architecture.md`: architecture and crate boundaries.
- `docs/dev/compatibility.md`: sticker pack format compatibility.
- `docs/user/README.md`: user-facing documentation index.
- `docs/agents/README.md`: agent handoff entrypoint.
- `docs/status/current.md`: current development state.
```

- [ ] **Step 2: Write `docs/dev/architecture.md`**

Create `docs/dev/architecture.md`:

```markdown
# Architecture

MSM is built as a Rust workspace. The domain crate owns MoreStickers compatibility types and provider-neutral logic. Later crates add storage, API, CLI, MCP, providers, and the final app binary.

## Crate Boundaries

- `msm-domain`: compatibility models, pure ID helpers, pure URL resolution, and import/export helpers.
- `msm-storage`: database repositories and asset storage, added in P2.
- `msm-api`: HTTP API and OpenAPI, added in P4.
- `msm-cli`: command-line client, added in P5.
- `msm-mcp`: MCP endpoint, added in P9.
- `msm-providers`: Telegram and LINE providers, added in P6.
- `msm-app`: final service binary and embedded frontend, added after API and Web UI foundations exist.

## Dependency Rule

`msm-domain` must not depend on Axum, SQLx, provider SDKs, frontend code, or runtime-specific infrastructure.
```

- [ ] **Step 3: Write `docs/dev/compatibility.md`**

Create `docs/dev/compatibility.md`:

```markdown
# MoreStickers Compatibility

MSM exports `.stickerpack` JSON compatible with Equicord moreStickers and MoreStickersConverter.

## Sticker

Required fields:

- `id`
- `image`
- `title`
- `stickerPackId`

Optional fields:

- `filename`
- `isAnimated`

## Sticker Pack

Required fields:

- `id`
- `title`
- `logo`
- `stickers`

Optional fields:

- `author`

## Provider ID Conventions

- Telegram pack: `MoreStickers:Telegram:Pack:{sticker_set_name}`
- Telegram sticker: `MoreStickers:Telegram:Sticker:{sticker_set_name}:{file_unique_id}`
- LINE sticker pack: `MoreStickers:Line:Pack:{id}`
- LINE sticker: `MoreStickers:Line:Sticker:{pack_id}:{sticker_id}`
- LINE emoji pack: `MoreStickers:Line:Emoji-Pack:{id}`
- LINE emoji: `MoreStickers:Line-Emoji:{pack_id}:{emoji_id}`

## Asset URL Rule

If the system public asset URL is configured, exported sticker images use that base URL. Otherwise they use the MSM public app URL.
```

- [ ] **Step 4: Write user documentation index**

Create `docs/user/README.md`:

```markdown
# MSM User Documentation

MSM is in P0/P1 foundation work. End-user setup and operation guides will be added as API, CLI, and Web UI phases are implemented.

Current usable contract: `.stickerpack` compatibility is documented in `../dev/compatibility.md`.
```

- [ ] **Step 5: Write agent entrypoint**

Create `docs/agents/README.md`:

```markdown
# Agent Handoff

Start here after any context loss.

Read in this order:

1. `../status/current.md`
2. `../status/decisions.md`
3. `project-map.md`
4. `compatibility.md`
5. `testing.md`
6. The active plan under `../superpowers/plans/`

Before stopping work, update `../status/current.md` and append a checkpoint to `../status/checkpoints.md`.
```

- [ ] **Step 6: Write agent project map**

Create `docs/agents/project-map.md`:

```markdown
# Project Map

## Implemented in P0/P1

- `crates/msm-domain`: MoreStickers-compatible domain models and pure helpers.
- `docs/status`: current state and development log.
- `docs/dev`: human developer references.
- `docs/agents`: progressive disclosure handoff docs.

## Not Implemented Yet

- API server.
- Database storage.
- Web UI.
- CLI.
- MCP endpoint.
- Provider network integrations.

Do not add cross-layer dependencies to `msm-domain`.
```

- [ ] **Step 7: Write agent compatibility guide**

Create `docs/agents/compatibility.md`:

```markdown
# Compatibility Guide For Agents

The external `.stickerpack` format is the stable contract. Internal MSM data may become richer later, but P1 exports must remain compatible.

When changing compatibility code:

1. Update or add a fixture under `crates/msm-domain/tests/fixtures/`.
2. Add a failing test in `crates/msm-domain/tests/compatibility.rs`.
3. Implement the smallest domain change.
4. Run `cargo test -p msm-domain`.
5. Update `docs/dev/compatibility.md` if the documented contract changes.
```

- [ ] **Step 8: Write status protocol**

Create `docs/agents/status-protocol.md`:

```markdown
# Status Protocol

Before pausing work:

1. Update `docs/status/current.md`.
2. Append one entry to `docs/status/checkpoints.md`.
3. Record the exact verification command and result.
4. Record any uncommitted files intentionally left behind.

Keep entries factual and short.
```

- [ ] **Step 9: Write testing guide**

Create `docs/agents/testing.md`:

```markdown
# Testing Guide

## P1 Domain Tests

Run:

```powershell
cargo test -p msm-domain
```

These tests prove:

- `.stickerpack` fixtures parse.
- serialized JSON uses camelCase field names.
- optional fields are skipped when absent.
- provider ID helpers match upstream conventions.
- asset URL resolution chooses CDN URL before app URL.

## Workspace Tests

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
```

- [ ] **Step 10: Write status files**

Create `docs/status/current.md`:

```markdown
# Current Status

Phase: P0/P1 foundation.

Last completed:
- Platform roadmap and foundation design was approved.

Current task:
- Implement repository hygiene, docs, Rust workspace, and `msm-domain` compatibility core.

Last verification:
- `git log --oneline -1 --format="%h %an <%ae> %s"` showed the design commit author as `Leko <leko@leko.moe>`.

Next step:
- Add repository hygiene and documentation baseline.

Known issues:
- PowerShell profile emits an fnm symlink permission warning in this environment.
```

Create `docs/status/decisions.md`:

```markdown
# Decisions

## 2026-05-02: Phase-Based Delivery

MSM is implemented in small phases. P0/P1 builds foundation and compatibility only. Later phases require their own specs and plans.

## 2026-05-02: Domain Crate Boundary

`msm-domain` owns compatibility models and pure helper logic. It does not depend on API, database, provider SDK, or frontend crates.

## 2026-05-02: External Format Stability

MoreStickers-compatible JSON is the external contract. Internal data may become richer later, but exports must preserve compatibility.
```

Create `docs/status/checkpoints.md`:

```markdown
# Checkpoints

## 2026-05-02

- Added and approved the MSM platform roadmap and P0/P1 foundation design.
- Started P0/P1 implementation planning.
```

- [ ] **Step 11: Verify docs exist**

Run:

```powershell
Test-Path README.md
Test-Path docs/dev/architecture.md
Test-Path docs/agents/README.md
Test-Path docs/status/current.md
```

Expected:

```text
True
True
True
True
```

- [ ] **Step 12: Commit documentation baseline**

Run:

```powershell
git add README.md docs/dev docs/user docs/agents docs/status
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "docs: add project handoff baseline"
```

Expected:

```text
[main <hash>] docs: add project handoff baseline
```

## Task 3: Rust Workspace Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `crates/msm-domain/Cargo.toml`
- Create: `crates/msm-domain/src/lib.rs`

- [ ] **Step 1: Write workspace manifest**

Create root `Cargo.toml`:

```toml
[workspace]
members = ["crates/msm-domain"]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/lekoOwO/MoreStickersManager-rs"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
```

- [ ] **Step 2: Write domain crate manifest**

Create `crates/msm-domain/Cargo.toml`:

```toml
[package]
name = "msm-domain"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
url = "2"

[lints]
workspace = true
```

- [ ] **Step 3: Write initial library file**

Create `crates/msm-domain/src/lib.rs`:

```rust
#![doc = "Domain models and pure helpers for MoreStickersManager-rs."]

pub mod error;
pub mod ids;
pub mod stickerpack;
pub mod url;

pub use error::{DomainError, DomainResult};
pub use ids::{
    line_emoji_id, line_emoji_pack_id, line_sticker_id, line_sticker_pack_id, telegram_pack_id,
    telegram_sticker_id,
};
pub use stickerpack::{
    Author, DynamicInfo, DynamicPackSetMeta, DynamicStickerPackMeta, Sticker, StickerPack,
};
pub use url::{AssetUrlConfig, AssetUrlInput, resolve_asset_url};
```

- [ ] **Step 4: Run check and observe expected failure**

Run:

```powershell
cargo test -p msm-domain
```

Expected failure:

```text
file not found for module `error`
file not found for module `ids`
file not found for module `stickerpack`
file not found for module `url`
```

- [ ] **Step 5: Commit workspace scaffold**

Run:

```powershell
git add Cargo.toml crates/msm-domain/Cargo.toml crates/msm-domain/src/lib.rs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "chore: scaffold Rust workspace"
```

Expected:

```text
[main <hash>] chore: scaffold Rust workspace
```

## Task 4: Domain Errors and Compatibility Models

**Files:**
- Create: `crates/msm-domain/src/error.rs`
- Create: `crates/msm-domain/src/stickerpack.rs`

- [ ] **Step 1: Write error module**

Create `crates/msm-domain/src/error.rs`:

```rust
use std::path::PathBuf;

pub type DomainResult<T> = Result<T, DomainError>;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("invalid sticker pack path extension: {path}")]
    InvalidStickerPackExtension { path: PathBuf },

    #[error("invalid provider id component `{component}`: {reason}")]
    InvalidProviderIdComponent {
        component: String,
        reason: &'static str,
    },

    #[error("invalid base URL `{url}`: {source}")]
    InvalidBaseUrl {
        url: String,
        source: url::ParseError,
    },
}
```

- [ ] **Step 2: Write compatibility models**

Create `crates/msm-domain/src/stickerpack.rs`:

```rust
use std::{collections::BTreeMap, fs, path::Path};

use crate::{DomainError, DomainResult};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sticker {
    pub id: String,
    pub image: String,
    pub title: String,
    pub sticker_pack_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_animated: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickerPack {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub logo: Sticker,
    pub stickers: Vec<Sticker>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub refresh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_headers: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicStickerPackMeta {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub logo: Sticker,
    pub dynamic: DynamicInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicPackSetMeta {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
    pub packs: Vec<DynamicStickerPackMeta>,
    pub refresh_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_headers: Option<BTreeMap<String, String>>,
}

impl StickerPack {
    pub fn from_json_str(input: &str) -> DomainResult<Self> {
        Ok(serde_json::from_str(input)?)
    }

    pub fn to_pretty_json(&self) -> DomainResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn read_stickerpack_file(path: impl AsRef<Path>) -> DomainResult<Self> {
        let path = path.as_ref();
        if path.extension().and_then(|value| value.to_str()) != Some("stickerpack") {
            return Err(DomainError::InvalidStickerPackExtension {
                path: path.to_path_buf(),
            });
        }

        Self::from_json_str(&fs::read_to_string(path).map_err(|error| {
            serde_json::Error::io(error)
        })?)
    }
}

impl DynamicPackSetMeta {
    pub fn from_json_str(input: &str) -> DomainResult<Self> {
        Ok(serde_json::from_str(input)?)
    }

    pub fn to_pretty_json(&self) -> DomainResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
```

- [ ] **Step 3: Run tests and observe expected failure**

Run:

```powershell
cargo test -p msm-domain
```

Expected failure:

```text
file not found for module `ids`
file not found for module `url`
```

- [ ] **Step 4: Commit models**

Run:

```powershell
git add crates/msm-domain/src/error.rs crates/msm-domain/src/stickerpack.rs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "feat: add sticker pack compatibility models"
```

Expected:

```text
[main <hash>] feat: add sticker pack compatibility models
```

## Task 5: Provider ID Helpers

**Files:**
- Create: `crates/msm-domain/src/ids.rs`
- Create: `crates/msm-domain/tests/compatibility.rs`

- [ ] **Step 1: Write failing provider ID tests**

Create `crates/msm-domain/tests/compatibility.rs` with the initial tests:

```rust
use msm_domain::{
    line_emoji_id, line_emoji_pack_id, line_sticker_id, line_sticker_pack_id, telegram_pack_id,
    telegram_sticker_id,
};

#[test]
fn telegram_ids_match_more_stickers_converter_conventions() {
    assert_eq!(
        telegram_pack_id("pack_name").unwrap(),
        "MoreStickers:Telegram:Pack:pack_name"
    );
    assert_eq!(
        telegram_sticker_id("pack_name", "file_unique_id").unwrap(),
        "MoreStickers:Telegram:Sticker:pack_name:file_unique_id"
    );
}

#[test]
fn line_sticker_ids_match_equicord_conventions() {
    assert_eq!(
        line_sticker_pack_id("12345").unwrap(),
        "MoreStickers:Line:Pack:12345"
    );
    assert_eq!(
        line_sticker_id("12345", "67890").unwrap(),
        "MoreStickers:Line:Sticker:12345:67890"
    );
}

#[test]
fn line_emoji_ids_match_equicord_conventions() {
    assert_eq!(
        line_emoji_pack_id("abcde").unwrap(),
        "MoreStickers:Line:Emoji-Pack:abcde"
    );
    assert_eq!(
        line_emoji_id("abcde", "fghij").unwrap(),
        "MoreStickers:Line-Emoji:abcde:fghij"
    );
}

#[test]
fn provider_ids_reject_empty_components() {
    assert!(telegram_pack_id("").is_err());
    assert!(telegram_sticker_id("pack", "").is_err());
    assert!(line_sticker_pack_id("").is_err());
    assert!(line_sticker_id("", "sticker").is_err());
    assert!(line_emoji_pack_id("").is_err());
    assert!(line_emoji_id("pack", "").is_err());
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
cargo test -p msm-domain telegram_ids_match_more_stickers_converter_conventions
```

Expected failure:

```text
file not found for module `ids`
```

- [ ] **Step 3: Implement provider ID helpers**

Create `crates/msm-domain/src/ids.rs`:

```rust
use crate::{DomainError, DomainResult};

fn validate_component(component: &str) -> DomainResult<()> {
    if component.is_empty() {
        return Err(DomainError::InvalidProviderIdComponent {
            component: component.to_owned(),
            reason: "component must not be empty",
        });
    }

    if component.contains(':') {
        return Err(DomainError::InvalidProviderIdComponent {
            component: component.to_owned(),
            reason: "component must not contain ':'",
        });
    }

    Ok(())
}

pub fn telegram_pack_id(sticker_set_name: &str) -> DomainResult<String> {
    validate_component(sticker_set_name)?;
    Ok(format!("MoreStickers:Telegram:Pack:{sticker_set_name}"))
}

pub fn telegram_sticker_id(sticker_set_name: &str, file_unique_id: &str) -> DomainResult<String> {
    validate_component(sticker_set_name)?;
    validate_component(file_unique_id)?;
    Ok(format!(
        "MoreStickers:Telegram:Sticker:{sticker_set_name}:{file_unique_id}"
    ))
}

pub fn line_sticker_pack_id(pack_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    Ok(format!("MoreStickers:Line:Pack:{pack_id}"))
}

pub fn line_sticker_id(pack_id: &str, sticker_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    validate_component(sticker_id)?;
    Ok(format!("MoreStickers:Line:Sticker:{pack_id}:{sticker_id}"))
}

pub fn line_emoji_pack_id(pack_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    Ok(format!("MoreStickers:Line:Emoji-Pack:{pack_id}"))
}

pub fn line_emoji_id(pack_id: &str, emoji_id: &str) -> DomainResult<String> {
    validate_component(pack_id)?;
    validate_component(emoji_id)?;
    Ok(format!("MoreStickers:Line-Emoji:{pack_id}:{emoji_id}"))
}
```

- [ ] **Step 4: Run provider ID tests**

Run:

```powershell
cargo test -p msm-domain provider_ids
```

Expected:

```text
test result: ok
```

- [ ] **Step 5: Commit provider ID helpers**

Run:

```powershell
git add crates/msm-domain/src/ids.rs crates/msm-domain/tests/compatibility.rs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "feat: add provider id helpers"
```

Expected:

```text
[main <hash>] feat: add provider id helpers
```

## Task 6: Asset URL Resolver

**Files:**
- Create: `crates/msm-domain/src/url.rs`
- Modify: `crates/msm-domain/tests/compatibility.rs`

- [ ] **Step 1: Add failing asset URL tests**

Append to `crates/msm-domain/tests/compatibility.rs`:

```rust
use msm_domain::{resolve_asset_url, AssetUrlConfig, AssetUrlInput};

#[test]
fn asset_url_uses_app_public_url_when_cdn_is_absent() {
    let config = AssetUrlConfig::new("https://msm.example").unwrap();
    let input = AssetUrlInput {
        pack_public_id: "pack_name",
        filename: "file_unique_id.webp",
    };

    assert_eq!(
        resolve_asset_url(&config, &input).unwrap(),
        "https://msm.example/assets/packs/pack_name/file_unique_id.webp"
    );
}

#[test]
fn asset_url_uses_public_asset_url_when_configured() {
    let config = AssetUrlConfig::new("https://msm.example")
        .unwrap()
        .with_public_asset_url("https://cdn.example/msm")
        .unwrap();
    let input = AssetUrlInput {
        pack_public_id: "pack_name",
        filename: "file_unique_id.webp",
    };

    assert_eq!(
        resolve_asset_url(&config, &input).unwrap(),
        "https://cdn.example/msm/assets/packs/pack_name/file_unique_id.webp"
    );
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
cargo test -p msm-domain asset_url_uses_app_public_url_when_cdn_is_absent
```

Expected failure:

```text
file not found for module `url`
```

- [ ] **Step 3: Implement URL resolver**

Create `crates/msm-domain/src/url.rs`:

```rust
use url::Url;

use crate::{DomainError, DomainResult};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetUrlConfig {
    public_app_url: Url,
    public_asset_url: Option<Url>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetUrlInput<'a> {
    pub pack_public_id: &'a str,
    pub filename: &'a str,
}

impl AssetUrlConfig {
    pub fn new(public_app_url: &str) -> DomainResult<Self> {
        Ok(Self {
            public_app_url: parse_base_url(public_app_url)?,
            public_asset_url: None,
        })
    }

    pub fn with_public_asset_url(mut self, public_asset_url: &str) -> DomainResult<Self> {
        self.public_asset_url = Some(parse_base_url(public_asset_url)?);
        Ok(self)
    }

    fn asset_base_url(&self) -> &Url {
        self.public_asset_url.as_ref().unwrap_or(&self.public_app_url)
    }
}

pub fn resolve_asset_url(config: &AssetUrlConfig, input: &AssetUrlInput<'_>) -> DomainResult<String> {
    let mut url = config.asset_base_url().clone();
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|()| DomainError::InvalidProviderIdComponent {
                component: url.to_string(),
                reason: "base URL cannot be a cannot-be-a-base URL",
            })?;
        segments.pop_if_empty();
        segments.extend(["assets", "packs", input.pack_public_id, input.filename]);
    }
    Ok(url.to_string())
}

fn parse_base_url(value: &str) -> DomainResult<Url> {
    Url::parse(value).map_err(|source| DomainError::InvalidBaseUrl {
        url: value.to_owned(),
        source,
    })
}
```

- [ ] **Step 4: Run URL tests**

Run:

```powershell
cargo test -p msm-domain asset_url
```

Expected:

```text
test result: ok
```

- [ ] **Step 5: Run formatter**

Run:

```powershell
cargo fmt --all
```

Expected: command exits with code 0 and may rewrite Rust formatting.

- [ ] **Step 6: Commit URL resolver**

Run:

```powershell
git add crates/msm-domain/src/url.rs crates/msm-domain/tests/compatibility.rs
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "feat: add asset URL resolver"
```

Expected:

```text
[main <hash>] feat: add asset URL resolver
```

## Task 7: Golden Fixtures and Roundtrip Tests

**Files:**
- Create: `crates/msm-domain/tests/fixtures/telegram_pack.stickerpack.json`
- Create: `crates/msm-domain/tests/fixtures/line_sticker_pack.stickerpack.json`
- Create: `crates/msm-domain/tests/fixtures/line_emoji_pack.stickerpack.json`
- Create: `crates/msm-domain/tests/fixtures/dynamic_pack_set.json`
- Modify: `crates/msm-domain/tests/compatibility.rs`

- [ ] **Step 1: Create Telegram fixture**

Create `crates/msm-domain/tests/fixtures/telegram_pack.stickerpack.json`:

```json
{
  "id": "MoreStickers:Telegram:Pack:sample_pack",
  "title": "Sample Telegram Pack",
  "logo": {
    "id": "MoreStickers:Telegram:Sticker:sample_pack:file_unique_1",
    "image": "https://msm.example/assets/packs/sample_pack/file_unique_1.webp",
    "title": "smile",
    "stickerPackId": "MoreStickers:Telegram:Pack:sample_pack",
    "filename": "file_unique_1.webp",
    "isAnimated": false
  },
  "stickers": [
    {
      "id": "MoreStickers:Telegram:Sticker:sample_pack:file_unique_1",
      "image": "https://msm.example/assets/packs/sample_pack/file_unique_1.webp",
      "title": "smile",
      "stickerPackId": "MoreStickers:Telegram:Pack:sample_pack",
      "filename": "file_unique_1.webp",
      "isAnimated": false
    }
  ]
}
```

- [ ] **Step 2: Create LINE sticker fixture**

Create `crates/msm-domain/tests/fixtures/line_sticker_pack.stickerpack.json`:

```json
{
  "id": "MoreStickers:Line:Pack:12345",
  "title": "Sample LINE Stickers",
  "author": {
    "name": "LINE Author",
    "url": "https://store.line.me/creators/author/en"
  },
  "logo": {
    "id": "MoreStickers:Line:Sticker:12345:67890",
    "image": "https://stickershop.line-scdn.net/stickershop/v1/sticker/67890/android/sticker.png",
    "title": "67890",
    "stickerPackId": "MoreStickers:Line:Pack:12345",
    "isAnimated": false
  },
  "stickers": [
    {
      "id": "MoreStickers:Line:Sticker:12345:67890",
      "image": "https://stickershop.line-scdn.net/stickershop/v1/sticker/67890/android/sticker.png",
      "title": "67890",
      "stickerPackId": "MoreStickers:Line:Pack:12345",
      "isAnimated": false
    }
  ]
}
```

- [ ] **Step 3: Create LINE emoji fixture**

Create `crates/msm-domain/tests/fixtures/line_emoji_pack.stickerpack.json`:

```json
{
  "id": "MoreStickers:Line:Emoji-Pack:abcde",
  "title": "Sample LINE Emoji",
  "author": {
    "name": "Emoji Author",
    "url": "https://store.line.me/emojis/author/en"
  },
  "logo": {
    "id": "MoreStickers:Line-Emoji:abcde:fghij",
    "image": "https://stickershop.line-scdn.net/sticonshop/v1/sticon/fghij/android/sticon.png",
    "title": "fghij",
    "stickerPackId": "MoreStickers:Line:Emoji-Pack:abcde",
    "isAnimated": true
  },
  "stickers": [
    {
      "id": "MoreStickers:Line-Emoji:abcde:fghij",
      "image": "https://stickershop.line-scdn.net/sticonshop/v1/sticon/fghij/android/sticon.png",
      "title": "fghij",
      "stickerPackId": "MoreStickers:Line:Emoji-Pack:abcde",
      "isAnimated": true
    }
  ]
}
```

- [ ] **Step 4: Create dynamic pack set fixture**

Create `crates/msm-domain/tests/fixtures/dynamic_pack_set.json`:

```json
{
  "id": "sub_sample",
  "version": "1",
  "title": "Sample Subscription",
  "author": {
    "name": "MSM"
  },
  "packs": [
    {
      "id": "MoreStickers:Telegram:Pack:sample_pack",
      "title": "Sample Telegram Pack",
      "logo": {
        "id": "MoreStickers:Telegram:Sticker:sample_pack:file_unique_1",
        "image": "https://msm.example/assets/packs/sample_pack/file_unique_1.webp",
        "title": "smile",
        "stickerPackId": "MoreStickers:Telegram:Pack:sample_pack",
        "filename": "file_unique_1.webp",
        "isAnimated": false
      },
      "dynamic": {
        "version": "1",
        "refreshUrl": "https://msm.example/api/public/packs/sample_pack/stickerpack",
        "authHeaders": {
          "Authorization": "Bearer sample"
        }
      }
    }
  ],
  "refreshUrl": "https://msm.example/api/public/subscriptions/sub_sample",
  "authHeaders": {
    "Authorization": "Bearer sample"
  }
}
```

- [ ] **Step 5: Add golden fixture tests**

Append to `crates/msm-domain/tests/compatibility.rs`:

```rust
use msm_domain::{DynamicPackSetMeta, StickerPack};

#[test]
fn telegram_fixture_roundtrips() {
    let input = include_str!("fixtures/telegram_pack.stickerpack.json");
    let pack = StickerPack::from_json_str(input).unwrap();

    assert_eq!(pack.id, "MoreStickers:Telegram:Pack:sample_pack");
    assert_eq!(pack.logo.sticker_pack_id, pack.id);
    assert_eq!(pack.stickers.len(), 1);

    let output = pack.to_pretty_json().unwrap();
    assert!(output.contains("\"stickerPackId\""));
    assert!(output.contains("\"isAnimated\""));
}

#[test]
fn line_sticker_fixture_roundtrips() {
    let input = include_str!("fixtures/line_sticker_pack.stickerpack.json");
    let pack = StickerPack::from_json_str(input).unwrap();

    assert_eq!(pack.id, "MoreStickers:Line:Pack:12345");
    assert_eq!(pack.author.unwrap().name, "LINE Author");
    assert_eq!(pack.stickers[0].sticker_pack_id, "MoreStickers:Line:Pack:12345");
}

#[test]
fn line_emoji_fixture_roundtrips() {
    let input = include_str!("fixtures/line_emoji_pack.stickerpack.json");
    let pack = StickerPack::from_json_str(input).unwrap();

    assert_eq!(pack.id, "MoreStickers:Line:Emoji-Pack:abcde");
    assert_eq!(pack.stickers[0].is_animated, Some(true));
}

#[test]
fn dynamic_pack_set_fixture_roundtrips() {
    let input = include_str!("fixtures/dynamic_pack_set.json");
    let pack_set = DynamicPackSetMeta::from_json_str(input).unwrap();

    assert_eq!(pack_set.id, "sub_sample");
    assert_eq!(pack_set.packs.len(), 1);
    assert_eq!(
        pack_set.packs[0].dynamic.refresh_url,
        "https://msm.example/api/public/packs/sample_pack/stickerpack"
    );

    let output = pack_set.to_pretty_json().unwrap();
    assert!(output.contains("\"refreshUrl\""));
    assert!(output.contains("\"authHeaders\""));
}

#[test]
fn optional_fields_are_skipped_when_absent() {
    let pack = StickerPack {
        id: "MoreStickers:Telegram:Pack:minimal".to_owned(),
        title: "Minimal".to_owned(),
        author: None,
        logo: msm_domain::Sticker {
            id: "MoreStickers:Telegram:Sticker:minimal:file".to_owned(),
            image: "https://msm.example/assets/packs/minimal/file.webp".to_owned(),
            title: "file".to_owned(),
            sticker_pack_id: "MoreStickers:Telegram:Pack:minimal".to_owned(),
            filename: None,
            is_animated: None,
        },
        stickers: Vec::new(),
    };

    let output = pack.to_pretty_json().unwrap();
    assert!(!output.contains("\"author\""));
    assert!(!output.contains("\"filename\""));
    assert!(!output.contains("\"isAnimated\""));
}
```

- [ ] **Step 6: Run golden tests**

Run:

```powershell
cargo test -p msm-domain
```

Expected:

```text
test result: ok
```

- [ ] **Step 7: Commit fixtures and tests**

Run:

```powershell
git add crates/msm-domain/tests
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "test: add sticker pack compatibility fixtures"
```

Expected:

```text
[main <hash>] test: add sticker pack compatibility fixtures
```

## Task 8: CI Baseline

**Files:**
- Create: `.github/workflows/ci.yml`
- Modify: `README.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] **Step 1: Create GitHub Actions workflow**

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches:
      - main
  pull_request:

jobs:
  rust:
    name: Rust
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Format
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Test
        run: cargo test --workspace
```

- [ ] **Step 2: Update README verification**

Ensure `README.md` contains this exact current baseline:

```markdown
## Development

Run the current baseline checks:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
```

- [ ] **Step 3: Run full verification**

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:

```text
cargo fmt exits 0
cargo clippy exits 0
cargo test exits 0
```

- [ ] **Step 4: Update status files**

Update `docs/status/current.md` to:

```markdown
# Current Status

Phase: P0/P1 foundation.

Last completed:
- Repository hygiene, documentation baseline, Rust workspace, `msm-domain` compatibility models, provider ID helpers, asset URL resolver, fixtures, and CI baseline.

Current task:
- Ready for review of P0/P1 implementation.

Last verification:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

Next step:
- Review P0/P1 implementation and decide whether to start P2 storage and asset core design.

Known issues:
- PowerShell profile emits an fnm symlink permission warning in this environment.
```

Append to `docs/status/checkpoints.md`:

```markdown

## 2026-05-02 P0/P1 Implementation

- Added repository hygiene and documentation baseline.
- Added Rust workspace and `msm-domain`.
- Added MoreStickers-compatible models, provider ID helpers, asset URL resolver, and golden tests.
- Added CI baseline.
- Verified workspace with format, clippy, and tests.
```

- [ ] **Step 5: Commit CI baseline**

Run:

```powershell
git add .github/workflows/ci.yml README.md docs/status/current.md docs/status/checkpoints.md
git -c user.name="Leko" -c user.email="leko@leko.moe" commit --author="Leko <leko@leko.moe>" -m "ci: add Rust verification workflow"
```

Expected:

```text
[main <hash>] ci: add Rust verification workflow
```

## Task 9: Final P0/P1 Review

**Files:**
- Modify: `docs/status/current.md` only if verification output differs from Task 8

- [ ] **Step 1: Inspect changed tracked files**

Run:

```powershell
git status --short --branch
```

Expected tracked state:

```text
## main
```

Acceptable untracked local files:

```text
?? .agents/
?? .claude/
?? package-lock.json
?? package.json
?? skills-lock.json
```

- [ ] **Step 2: Verify commit authors**

Run:

```powershell
git log --format="%h %an <%ae> %s" --max-count=10
```

Expected: every implementation commit author is:

```text
Leko <leko@leko.moe>
```

- [ ] **Step 3: Run final commands**

Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected:

```text
all commands exit 0
```

- [ ] **Step 4: Record review result**

If final commands pass, update `docs/status/current.md` only if its verification section is inaccurate.

If a command fails, update `docs/status/current.md` with:

```markdown
Known issues:
- `<command>` failed with `<short failure reason>`.
```

Then fix the failure before claiming P0/P1 complete.

## Self-Review Checklist

- Every spec requirement in P0 has a task: repo hygiene, docs, status, agent handoff, CI baseline.
- Every spec requirement in P1 has a task: compatibility models, import/export helpers, fixtures, provider IDs, URL resolution, tests.
- No API, DB, Web UI, CLI, MCP, or provider network work is included.
- Commit commands use `Leko <leko@leko.moe>` and no tool authorship labels.
- Verification commands are explicit and repeatable.
