# MSM P16 Web PAT Management Design

## Scope

P16 lets the Web UI store a Personal Access Token locally, send it to protected
API calls, and manage PAT lifecycle endpoints from the browser.

## Goals

- Add Web API client support for Bearer PAT headers.
- Add typed Web PAT client methods for create, list, and revoke.
- Add a Web UI token panel backed by `localStorage`.
- Pass the configured token into the pack dashboard API client.
- Add a PAT management panel that can list, create, and revoke PATs.
- Keep mock pack behavior when no API base URL is configured.

## Non-Goals

- No secure server-side session storage.
- No OIDC/local-login UI.
- No role/admin UI.
- No Web routing.

## UX Contract

The Web UI treats PATs as local browser configuration:

- `VITE_MSM_PAT` can seed the initial token for development.
- users can paste a PAT into the UI and store it in `localStorage`;
- create responses show the raw token once in the UI;
- list responses never show token hashes.
