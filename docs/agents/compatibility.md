# Compatibility Guide For Agents

The external `.stickerpack` format is the stable contract. Internal MSM data may become richer later, but P1 exports must remain compatible.

When changing compatibility code:

1. Update or add a fixture under `crates/msm-domain/tests/fixtures/`.
2. Add a failing test in `crates/msm-domain/tests/compatibility.rs`.
3. Implement the smallest domain change.
4. Run `cargo test -p msm-domain`.
5. Update `docs/dev/compatibility.md` if the documented contract changes.
