# MSM P19 Web Local Login Design

## Scope

P19 connects the P18 local auth bootstrap APIs to the Web UI.

## Goals

- Add Web API client methods for local register and login.
- Add a Web local auth panel.
- Store the login-issued PAT through the existing browser-local PAT path.
- Keep the raw login token visible once after login.
- Keep the existing manual PAT panel for CLI/MCP tokens.

## Non-Goals

- No cookie sessions.
- No route guards.
- No password reset.
- No OIDC UI.

## UX Contract

The Web UI shows a local auth card with email, password, user ID, display name,
token ID, and scopes. Register creates the local user. Login creates a PAT and
stores it in `localStorage` key `msm.pat` through the existing token flow.
