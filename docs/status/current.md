# Current Status

Phase: P21 pack CRUD foundation.

Last completed:
- P21 pack CRUD foundation implementation: owned pack rename/visibility update and delete across storage, API, CLI, and MCP.

Current task:
- P21 is verified. Commit P21, then continue with Web UI pack CRUD controls or folder/tag/subscription-group management.

Last verification:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked`
- `npm run web:typecheck`
- `npm run web:test`
- `npm run web:build`
- `cargo test -p msm-storage --locked`
- `cargo test -p msm-api --locked`
- `cargo test -p msm-cli --locked`
- `cargo test -p msm-mcp --locked`

Next step:
- Commit P21 with author `Leko <leko@leko.moe>`.

Known issues:
- PowerShell profile emits an fnm symlink permission warning in this environment.
- Docker CLI is not installed in this environment, so Docker image build is deferred to GitHub Actions or a Docker-enabled machine.
- `skills-lock.json` is an existing untracked local skill lock file and is not part of P7.
