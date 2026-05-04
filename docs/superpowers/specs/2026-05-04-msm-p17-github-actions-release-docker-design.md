# MSM P17 GitHub Actions Release And Docker Design

## Scope

P17 aligns MSM GitHub Actions with the referenced `lekoOwO/github-stars-scraper`
workflow set:

- `ci.yml`: validate pushes and pull requests.
- `docker.yml`: build and publish a multi-arch GHCR image.
- `prerelease.yml`: publish a moving `prerelease` GitHub Release from `main`.
- `release.yml`: publish GitHub Release artifacts for `v*` tags.

## Reference Summary

The reference workflows use:

- `actions/checkout@v4`;
- `dtolnay/rust-toolchain@stable`;
- `Swatinem/rust-cache@v2`;
- `docker/setup-qemu-action`, `docker/setup-buildx-action`, `docker/login-action`,
  `docker/metadata-action`, and `docker/build-push-action`;
- `taiki-e/install-action` for `cross`;
- `actions/upload-artifact`, `actions/download-artifact`;
- `softprops/action-gh-release`.

## MSM Adaptation

MSM is Rust + Vue, so binary release and Docker build steps must build the Web
dist before compiling `msm-app`. The Rust binary artifact name is `msm-app` on
Unix targets and `msm-app.exe` on Windows targets.

## Goals

- Keep CI fast and representative of local verification.
- Build frontend dist before release binaries so P10 embeds the real Web UI.
- Publish Linux, macOS, and Windows binary artifacts.
- Publish Docker images to `ghcr.io/${{ github.repository }}`.

## Non-Goals

- No deployment workflow.
- No package manager other than npm.
- No Docker runtime database bootstrap beyond existing MSM environment
  variables.
