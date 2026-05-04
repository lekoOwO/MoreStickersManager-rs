# MSM P20 Admin Bootstrap Policy Design

## Scope

P20 extends local registration with an optional tenant bootstrap. A new local
user can create an initial tenant and become its admin in the same request.

## Goals

- Add optional fields to local register:
  - `tenantId`
  - `tenantName`
  - `tenantRole`
- When `tenantId` is present, create the tenant if needed and add the registered
  user as a tenant member.
- Default `tenantName` to `tenantId`.
- Default `tenantRole` to `admin`.
- Keep registration without tenant fields working.

## Non-Goals

- No invitation flow.
- No tenant member management UI.
- No open registration policy enforcement.
- No "first user only" lockout yet.
