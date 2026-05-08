# Agent Handoff

Start here after any context loss. Keep context small and prefer the living PRD
over historical phase notes.

## Read Order

1. `../PRD.md`: product requirements, current status, roadmap, acceptance rules.
2. `../status/current.md`: latest session state and recent verification.
3. `../status/checkpoints.md`: chronological progress log when more detail is
   needed.
4. `../dev/architecture.md`: crate boundaries and runtime architecture.
5. `../dev/compatibility.md`: MoreStickers format contract.
6. `../user/README.md`: user-facing commands and routes.

## Update Protocol

Before stopping work:

1. Update `../PRD.md` if capability status, roadmap, or acceptance criteria
   changed.
2. Update `../status/current.md` with the latest completed slice, verification,
   next step, and known issues.
3. Update `../status/implementation-matrix.md` when feature status changes.
4. Append one short entry to `../status/checkpoints.md`.
5. Record exact verification commands and results.
6. Leave intentionally untracked local files unmodified and mention them in the
   final handoff.

## Development Rules

- Preserve MoreStickers `.stickerpack` compatibility.
- Keep Web/API/CLI/MCP parity visible in `../PRD.md`.
- Use TDD for behavior changes.
- Use installed Microsoft Edge for E2E; do not install Chromium unless the user
  explicitly asks.
- Do not add tool, assistant, bot, or generated-by attribution to commits,
  branches, PR titles, PR bodies, or release text.
