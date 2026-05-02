# MSM P3 Authorization Domain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add pure, tested authorization policy primitives to `msm-domain`.

**Architecture:** Keep policy evaluation provider-neutral and storage-free. Later API, CLI, MCP, and Web UI layers call these pure functions with resource snapshots loaded from storage.

**Tech Stack:** Rust enums/structs, deterministic unit tests, no new runtime dependencies.

---

## Task 1: Add Authorization Module and Tests

**Files:**
- Modify: `crates/msm-domain/src/lib.rs`
- Create: `crates/msm-domain/src/authz.rs`
- Create: `crates/msm-domain/tests/authorization.rs`

- [ ] Add `authz` module to `msm-domain`.
- [ ] Define `Permission`, `Role`, `Principal`, `Visibility`, `MemberAccess`, `PackAction`, `SubscriptionAction`, `PackResource`, `SubscriptionResource`, `AccessContext`, `PolicyDecision`, and `PolicyReason`.
- [ ] Implement built-in role permission checks.
- [ ] Implement `evaluate_pack_access`.
- [ ] Implement `evaluate_subscription_access`.
- [ ] Add tests for admin, owner, member, anonymous, PAT, pack secret, and subscription secret cases.
- [ ] Run `cargo test -p msm-domain authorization`.
- [ ] Commit with `feat: add domain authorization policies`.

## Task 2: Update Documentation and Status

**Files:**
- Modify: `docs/dev/architecture.md`
- Modify: `docs/agents/project-map.md`
- Modify: `docs/agents/testing.md`
- Modify: `docs/status/current.md`
- Modify: `docs/status/checkpoints.md`

- [ ] Document `msm-domain::authz`.
- [ ] Add authorization test command to agent testing docs.
- [ ] Update status files with P3 result.
- [ ] Run:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

- [ ] Commit with `docs: update P3 authorization status`.

## Completion Criteria

- Authorization policy tests pass.
- Workspace format, clippy, and test checks pass.
- Policy code does not depend on storage, API, frontend, or provider crates.
