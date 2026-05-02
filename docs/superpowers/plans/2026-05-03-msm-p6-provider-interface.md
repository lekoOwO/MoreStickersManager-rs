# MSM P6 Provider Interface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add provider abstractions and fixture-driven Telegram/LINE normalization.

**Architecture:** Create `msm-providers` as a pure normalization crate depending on `msm-domain`.

**Tech Stack:** Rust, serde, serde_json, msm-domain.

---

## Task 1: Scaffold Providers Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/msm-providers/Cargo.toml`
- Create: `crates/msm-providers/src/lib.rs`
- Create: `crates/msm-providers/src/error.rs`
- Create: `crates/msm-providers/src/registry.rs`

- [ ] Add `crates/msm-providers` to workspace.
- [ ] Add dependencies: `msm-domain`, `serde`, `serde_json`, `thiserror`, `url`.
- [ ] Add provider metadata, capabilities, status, and typed errors.
- [ ] Add registry with implemented and planned providers.
- [ ] Run `cargo test -p msm-providers registry`.
- [ ] Commit with `chore: scaffold providers crate`.

## Task 2: Telegram Normalizer

**Files:**
- Create: `crates/msm-providers/src/telegram.rs`
- Modify: `crates/msm-providers/src/lib.rs`

- [ ] Implement Telegram fixture input structs.
- [ ] Implement `TelegramProvider`.
- [ ] Add tests for ID and URL output.
- [ ] Commit with `feat: add Telegram provider normalizer`.

## Task 3: LINE Normalizers

**Files:**
- Create: `crates/msm-providers/src/line.rs`
- Modify: `crates/msm-providers/src/lib.rs`

- [ ] Implement LINE sticker and LINE emoji fixture input structs.
- [ ] Implement `LineStickerProvider` and `LineEmojiProvider`.
- [ ] Add tests for Equicord-compatible IDs.
- [ ] Commit with `feat: add LINE provider normalizers`.

## Task 4: Docs and Verification

**Files:**
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] Document `msm-providers`.
- [ ] Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] Commit with `docs: update P6 provider status`.

## Completion Criteria

- Provider registry tests pass.
- Telegram/LINE normalization tests pass.
- Workspace verification passes.
