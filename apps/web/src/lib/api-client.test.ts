import { describe, expect, it, vi } from "vitest";

import {
  createPackClient,
  createLocalAuthClient,
  createPatClient,
  createProductMetadataClient,
  createTenantAdminClient,
  folderListUrl,
  folderPackListUrl,
  mapApiPackRecord,
  packListUrl,
  packTagListUrl,
  patScopePolicyUrl,
  subscriptionGroupPackListUrl,
  subscriptionGroupListUrl,
  subscriptionLinkListUrl,
  tagListUrl,
  tenantMemberListUrl,
  tenantMemberUrl,
  tenantOidcProviderUrl,
  tenantOidcProvidersUrl,
  tenantRoleUrl,
  tenantRolesUrl,
  tenantSettingsUrl,
  tenantUserStatusUrl,
  type ApiStickerPackRecord,
} from "./api-client";

function jsonResponse(body: unknown, status = 200) {
  return new Response(body === null ? null : JSON.stringify(body), { status });
}

describe("pack API client", () => {
  it("uses mock packs when no API base URL is configured", async () => {
    const client = createPackClient();

    const packs = await client.listStickerPacks();

    expect(packs).toHaveLength(3);
    expect(packs[0]?.provider).toBe("Telegram");
  });

  it("constructs the P4 pack list URL with encoded user ID", () => {
    expect(packListUrl("https://msm.example.test/", "user 1@example.test")).toBe(
      "https://msm.example.test/api/v1/packs?userId=user+1%40example.test",
    );
  });

  it("fetches and maps API records", async () => {
    const fetchImpl = vi.fn(async () => {
      return new Response(
        JSON.stringify([
          {
            id: "pack_1",
            title: "API Cats",
            visibility: "Public",
            compatibility_id: "MoreStickers:Telegram:Pack:api_cats",
            sticker_pack: {
              id: "MoreStickers:Telegram:Pack:api_cats",
              title: "API Cats",
              stickers: [{ id: "sticker_1" }, { id: "sticker_2" }],
            },
            updated_at: "2026-05-03T01:02:03Z",
          },
        ] satisfies ApiStickerPackRecord[]),
        { status: 200 },
      );
    });

    const client = createPackClient({
      baseUrl: "https://msm.example.test",
      userId: "user_1",
      fetchImpl,
    });

    const packs = await client.listStickerPacks();

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/packs?userId=user_1");
    expect(packs).toEqual([
      {
        id: "pack_1",
        title: "API Cats",
        provider: "Telegram",
        visibility: "public",
        stickerCount: 2,
        subscriptionReady: true,
        updatedAt: "2026-05-03",
      },
    ]);
  });

  it("sends bearer auth when listing packs with a configured PAT", async () => {
    const fetchImpl = vi.fn(async () => new Response(JSON.stringify([]), { status: 200 }));
    const client = createPackClient({
      baseUrl: "https://msm.example.test",
      userId: "user_1",
      authToken: "msm_pat_cli1_secret",
      fetchImpl,
    });

    await client.listStickerPacks();

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/packs?userId=user_1", {
      headers: {
        Authorization: "Bearer msm_pat_cli1_secret",
      },
    });
  });

  it("updates and deletes sticker packs with bearer auth", async () => {
    const fetchImpl = vi.fn(async () => new Response(JSON.stringify({ id: "pack_1" }), { status: 200 }));
    const client = createPackClient({
      baseUrl: "https://msm.example.test/",
      userId: "user_1",
      authToken: "msm_pat_cli1_secret",
      fetchImpl,
    });

    await client.updateStickerPack({
      packId: "pack_1",
      title: "Renamed Pack",
      visibility: "public",
    });
    await client.deleteStickerPack("pack_1");

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/packs/pack_1", {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_cli1_secret",
      },
      body: JSON.stringify({
        title: "Renamed Pack",
        visibility: "public",
      }),
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/packs/pack_1", {
      method: "DELETE",
      headers: {
        Authorization: "Bearer msm_pat_cli1_secret",
      },
    });
  });

  it("imports sticker packs with bearer auth", async () => {
    const fetchImpl = vi.fn(async () => new Response(null, { status: 201 }));
    const client = createPackClient({
      baseUrl: "https://msm.example.test/",
      userId: "user_1",
      authToken: "msm_pat_cli1_secret",
      fetchImpl,
    });
    const pack = {
      id: "MoreStickers:Telegram:Pack:cats",
      title: "Cats",
      logo: {
        id: "sticker_1",
        title: "cat",
        image: "https://msm.example/cat.webp",
        sticker_pack_id: "MoreStickers:Telegram:Pack:cats",
      },
      stickers: [],
    };

    await client.importStickerPack({
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      packId: "pack_1",
      visibility: "private",
      pack,
    });

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/packs/import", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_cli1_secret",
      },
      body: JSON.stringify({
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        packId: "pack_1",
        visibility: "private",
        pack,
      }),
    });
  });

  it("infers LINE emoji provider and private visibility from API record", () => {
    const summary = mapApiPackRecord({
      id: "pack_2",
      visibility: "Private",
      compatibilityId: "MoreStickers:Line:Emoji-Pack:emoji_cats",
      stickerPack: {
        id: "MoreStickers:Line:Emoji-Pack:emoji_cats",
        title: "Emoji Cats",
        stickers: [{}],
      },
    });

    expect(summary.provider).toBe("LINE Emojis");
    expect(summary.visibility).toBe("private");
    expect(summary.stickerCount).toBe(1);
  });
});

describe("tenant admin API client", () => {
  it("constructs tenant member URLs with encoded path segments", () => {
    expect(tenantMemberListUrl("https://msm.example.test/", "tenant 1")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/members",
    );
    expect(tenantMemberUrl("https://msm.example.test/", "tenant 1", "user 2")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/members/user%202",
    );
    expect(tenantSettingsUrl("https://msm.example.test/", "tenant 1")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/settings",
    );
    expect(tenantUserStatusUrl("https://msm.example.test/", "tenant 1", "user 2")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/users/user%202/status",
    );
    expect(tenantRolesUrl("https://msm.example.test/", "tenant 1")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/roles",
    );
    expect(tenantRoleUrl("https://msm.example.test/", "tenant 1", "role editor")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/roles/role%20editor",
    );
    expect(tenantOidcProvidersUrl("https://msm.example.test/", "tenant 1")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/oidc-providers",
    );
    expect(tenantOidcProviderUrl("https://msm.example.test/", "tenant 1", "google workspace")).toBe(
      "https://msm.example.test/api/v1/tenants/tenant%201/oidc-providers/google%20workspace",
    );
  });

  it("lists and updates tenant member roles with bearer auth", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (init?.method === "PUT") {
        return new Response(
          JSON.stringify({
            tenantId: "tenant_1",
            userId: "user_2",
            role: "admin",
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 200 },
        );
      }

      return new Response(
        JSON.stringify([
          {
            tenantId: "tenant_1",
            userId: "user_1",
            role: "admin",
            createdAt: "2026-05-09T00:00:00Z",
          },
          {
            tenantId: "tenant_1",
            userId: "user_2",
            role: "user",
            createdAt: "2026-05-09T00:00:00Z",
          },
        ]),
        { status: 200 },
      );
    });
    const client = createTenantAdminClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_admin_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const members = await client.listTenantMembers("tenant_1");
    const updated = await client.setTenantMemberRole("tenant_1", "user_2", "admin");

    expect(members).toHaveLength(2);
    expect(updated.role).toBe("admin");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/members", {
      headers: {
        Authorization: "Bearer msm_pat_admin_secret",
      },
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/members/user_2", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_admin_secret",
      },
      body: JSON.stringify({ role: "admin" }),
    });
  });

  it("manages tenant settings, user status, and role templates with bearer auth", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith("/settings") && init?.method === "PUT") {
        return new Response(
          JSON.stringify({
            tenantId: "tenant_1",
            name: "Production",
            publicAssetUrl: "https://cdn.example.test/msm",
            localRegistrationEnabled: false,
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 200 },
        );
      }
      if (url.endsWith("/settings")) {
        return new Response(
          JSON.stringify({
            tenantId: "tenant_1",
            name: "Default tenant",
            publicAssetUrl: null,
            localRegistrationEnabled: true,
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 200 },
        );
      }
      if (url.endsWith("/users/user_2/status")) {
        return new Response(
          JSON.stringify({
            id: "user_2",
            email: "member@example.com",
            displayName: "Member",
            isDisabled: true,
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 200 },
        );
      }
      if (url.endsWith("/roles/role_editor")) {
        return new Response(
          JSON.stringify({
            id: "role_editor",
            tenantId: "tenant_1",
            name: "Editors",
            permissions: ["pack.read", "pack.update"],
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 200 },
        );
      }

      return new Response(
        JSON.stringify([
          {
            id: "role_editor",
            tenantId: "tenant_1",
            name: "Editors",
            permissions: ["pack.read", "pack.update"],
            createdAt: "2026-05-09T00:00:00Z",
          },
        ]),
        { status: 200 },
      );
    });
    const client = createTenantAdminClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_admin_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const settings = await client.getTenantSettings("tenant_1");
    const updatedSettings = await client.updateTenantSettings("tenant_1", {
      name: "Production",
      publicAssetUrl: "https://cdn.example.test/msm",
      localRegistrationEnabled: false,
    });
    const user = await client.setTenantUserStatus("tenant_1", "user_2", true);
    const role = await client.upsertTenantRole("tenant_1", "role_editor", {
      name: "Editors",
      permissions: ["pack.read", "pack.update"],
    });
    const roles = await client.listTenantRoles("tenant_1");

    expect(settings.name).toBe("Default tenant");
    expect(updatedSettings.publicAssetUrl).toBe("https://cdn.example.test/msm");
    expect(updatedSettings.localRegistrationEnabled).toBe(false);
    expect(user.isDisabled).toBe(true);
    expect(role.permissions).toEqual(["pack.read", "pack.update"]);
    expect(roles[0]?.id).toBe("role_editor");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/settings", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_admin_secret",
      },
      body: JSON.stringify({
        name: "Production",
        publicAssetUrl: "https://cdn.example.test/msm",
        localRegistrationEnabled: false,
      }),
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/users/user_2/status", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_admin_secret",
      },
      body: JSON.stringify({ isDisabled: true }),
    });
  });

  it("manages OIDC providers with bearer auth", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith("/oidc-providers/google") && init?.method === "PUT") {
        return jsonResponse({
          id: "google",
          tenantId: "tenant_1",
          displayName: "Google Workspace",
          issuerUrl: "https://accounts.google.com",
          clientId: "client_1",
          clientSecret: "[redacted]",
          scopes: ["openid", "email"],
          isEnabled: true,
          allowRegistration: false,
          createdAt: "2026-05-10T00:00:00Z",
          updatedAt: "2026-05-10T00:00:00Z",
        });
      }
      if (url.endsWith("/oidc-providers/google") && init?.method === "DELETE") {
        return new Response(null === null ? null : JSON.stringify(null), { status: 204 });
      }
      if (url.endsWith("/oidc-providers")) {
        return jsonResponse([
          {
            id: "google",
            tenantId: "tenant_1",
            displayName: "Google Workspace",
            issuerUrl: "https://accounts.google.com",
            clientId: "client_1",
            clientSecret: "[redacted]",
            scopes: ["openid", "email"],
            isEnabled: true,
            allowRegistration: false,
            createdAt: "2026-05-10T00:00:00Z",
            updatedAt: "2026-05-10T00:00:00Z",
          },
        ]);
      }
      throw new Error(`Unexpected request: ${url}`);
    }) as unknown as typeof fetch;
    const client = createTenantAdminClient({
      baseUrl: "https://msm.example.test",
      authToken: "msm_pat_admin_secret",
      fetchImpl,
    });

    const providers = await client.listOidcProviders("tenant_1");
    const upserted = await client.upsertOidcProvider("tenant_1", "google", {
      displayName: "Google Workspace",
      issuerUrl: "https://accounts.google.com",
      clientId: "client_1",
      clientSecret: "secret_1",
      scopes: ["openid", "email"],
      isEnabled: true,
      allowRegistration: false,
    });
    await client.deleteOidcProvider("tenant_1", "google");

    expect(providers[0]?.id).toBe("google");
    expect(upserted.clientSecret).toBe("[redacted]");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/oidc-providers", {
      headers: { Authorization: "Bearer msm_pat_admin_secret" },
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/oidc-providers/google", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_admin_secret",
      },
      body: JSON.stringify({
        displayName: "Google Workspace",
        issuerUrl: "https://accounts.google.com",
        clientId: "client_1",
        clientSecret: "secret_1",
        scopes: ["openid", "email"],
        isEnabled: true,
        allowRegistration: false,
      }),
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/tenants/tenant_1/oidc-providers/google", {
      method: "DELETE",
      headers: { Authorization: "Bearer msm_pat_admin_secret" },
    });
  });
});

describe("PAT API client", () => {
  it("creates a PAT and returns the raw token from the create response", async () => {
    const fetchImpl = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          id: "cli1",
          userId: "user_1",
          name: "CLI",
          token: "msm_pat_cli1_secret",
          scopes: ["pack.read"],
          expiresAt: null,
          revokedAt: null,
          createdAt: "2026-05-04T00:00:00Z",
        }),
        { status: 201 },
      );
    });
    const client = createPatClient({
      baseUrl: "https://msm.example.test",
      authToken: "msm_pat_admin_secret",
      fetchImpl,
    });

    const created = await client.createPersonalAccessToken({
      id: "cli1",
      userId: "user_1",
      name: "CLI",
      scopes: ["pack.read"],
      expiresAt: null,
    });

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/pats", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_admin_secret",
      },
      body: JSON.stringify({
        id: "cli1",
        userId: "user_1",
        name: "CLI",
        scopes: ["pack.read"],
        expiresAt: null,
      }),
    });
    expect(created.token).toBe("msm_pat_cli1_secret");
  });

  it("lists and revokes PATs", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (init?.method === "DELETE") {
        return new Response(null, { status: 204 });
      }

      return new Response(
        JSON.stringify([
          {
            id: "cli1",
            userId: "user_1",
            name: "CLI",
            scopes: ["pack.read"],
            expiresAt: null,
            revokedAt: null,
            createdAt: "2026-05-04T00:00:00Z",
          },
        ]),
        { status: 200 },
      );
    });
    const client = createPatClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_admin_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const tokens = await client.listPersonalAccessTokens("user_1");
    await client.revokePersonalAccessToken("cli1");

    expect(tokens[0]?.id).toBe("cli1");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/pats?userId=user_1", {
      headers: {
        Authorization: "Bearer msm_pat_admin_secret",
      },
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/pats/cli1", {
      method: "DELETE",
      headers: {
        Authorization: "Bearer msm_pat_admin_secret",
      },
    });
  });

  it("loads PAT scope policy with bearer auth", async () => {
    const fetchImpl = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          userId: "user_1",
          allowedScopes: ["pack.read", "pat.manage"],
        }),
        { status: 200 },
      );
    });
    const client = createPatClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_admin_secret",
      fetchImpl,
    });

    const policy = await client.getPatScopePolicy("user_1");

    expect(policy.allowedScopes).toEqual(["pack.read", "pat.manage"]);
    expect(fetchImpl).toHaveBeenCalledWith(
      patScopePolicyUrl("https://msm.example.test/", "user_1"),
      {
        headers: {
          Authorization: "Bearer msm_pat_admin_secret",
        },
      },
    );
  });
});

describe("local auth API client", () => {
  it("registers a local user", async () => {
    const fetchImpl = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          id: "user_1",
          email: "leko@example.com",
          displayName: "Leko",
        }),
        { status: 201 },
      );
    });
    const client = createLocalAuthClient({
      baseUrl: "https://msm.example.test",
      fetchImpl,
    });

    const user = await client.registerLocalUser({
      id: "user_1",
      email: "leko@example.com",
      displayName: "Leko",
      password: "password",
    });

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/auth/local/register", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        id: "user_1",
        email: "leko@example.com",
        displayName: "Leko",
        password: "password",
      }),
    });
    expect(user.id).toBe("user_1");
  });

  it("logs in and returns a raw PAT", async () => {
    const fetchImpl = vi.fn(async () => {
      return new Response(
        JSON.stringify({
          id: "webui",
          userId: "user_1",
          name: "Web UI",
          token: "msm_pat_webui_secret",
          scopes: ["pack.read"],
          expiresAt: null,
          revokedAt: null,
          createdAt: "2026-05-04T00:00:00Z",
        }),
        { status: 200 },
      );
    });
    const client = createLocalAuthClient({
      baseUrl: "https://msm.example.test/",
      fetchImpl,
    });

    const login = await client.loginLocalUser({
      email: "leko@example.com",
      password: "password",
      tokenId: "webui",
      tokenName: "Web UI",
      scopes: ["pack.read"],
      expiresAt: null,
    });

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/auth/local/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        email: "leko@example.com",
        password: "password",
        tokenId: "webui",
        tokenName: "Web UI",
        scopes: ["pack.read"],
        expiresAt: null,
      }),
    });
    expect(login.token).toBe("msm_pat_webui_secret");
  });
});

describe("product metadata API client", () => {
  it("constructs metadata list URLs with encoded tenant and owner IDs", () => {
    expect(folderListUrl("https://msm.example.test/", "tenant 1", "user 1")).toBe(
      "https://msm.example.test/api/v1/folders?tenantId=tenant+1&ownerUserId=user+1",
    );
    expect(tagListUrl("https://msm.example.test/", "tenant 1")).toBe(
      "https://msm.example.test/api/v1/tags?tenantId=tenant+1",
    );
    expect(subscriptionGroupListUrl("https://msm.example.test/", "tenant 1", "user 1")).toBe(
      "https://msm.example.test/api/v1/subscription-groups?tenantId=tenant+1&ownerUserId=user+1",
    );
    expect(subscriptionLinkListUrl("https://msm.example.test/", "user 1")).toBe(
      "https://msm.example.test/api/v1/subscription-access-tokens?userId=user+1",
    );
    expect(folderPackListUrl("https://msm.example.test/", "folder 1")).toBe(
      "https://msm.example.test/api/v1/folders/folder%201/packs",
    );
    expect(packTagListUrl("https://msm.example.test/", "pack 1")).toBe(
      "https://msm.example.test/api/v1/packs/pack%201/tags",
    );
    expect(subscriptionGroupPackListUrl("https://msm.example.test/", "sub 1")).toBe(
      "https://msm.example.test/api/v1/subscription-groups/sub%201/packs",
    );
  });

  it("lists and creates folders, tags, and subscription groups with bearer auth", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (init?.method === "POST" && url.endsWith("/api/v1/folders")) {
        return new Response(
          JSON.stringify({
            id: "folder_1",
            tenantId: "tenant_1",
            ownerUserId: "user_1",
            name: "Favorites",
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 201 },
        );
      }
      if (init?.method === "POST" && url.endsWith("/api/v1/tags")) {
        return new Response(
          JSON.stringify({ id: "tag_1", tenantId: "tenant_1", name: "cute", createdAt: "2026-05-09T00:00:00Z" }),
          { status: 201 },
        );
      }
      if (init?.method === "POST" && url.endsWith("/api/v1/subscription-groups")) {
        return new Response(
          JSON.stringify({
            id: "sub_1",
            tenantId: "tenant_1",
            ownerUserId: "user_1",
            title: "Weekly",
            visibility: "private",
            createdAt: "2026-05-09T00:00:00Z",
          }),
          { status: 201 },
        );
      }
      if (url.includes("/api/v1/folders?")) {
        return new Response(
          JSON.stringify([
            {
              id: "folder_1",
              tenantId: "tenant_1",
              ownerUserId: "user_1",
              name: "Favorites",
              createdAt: "2026-05-09T00:00:00Z",
            },
          ]),
          { status: 200 },
        );
      }
      if (url.includes("/api/v1/tags?")) {
        return new Response(
          JSON.stringify([{ id: "tag_1", tenantId: "tenant_1", name: "cute", createdAt: "2026-05-09T00:00:00Z" }]),
          { status: 200 },
        );
      }
      return new Response(
        JSON.stringify([
          {
            id: "sub_1",
            tenantId: "tenant_1",
            ownerUserId: "user_1",
            title: "Weekly",
            visibility: "private",
            createdAt: "2026-05-09T00:00:00Z",
          },
        ]),
        { status: 200 },
      );
    });
    const client = createProductMetadataClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_web_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const folder = await client.createFolder({
      id: "folder_1",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      name: "Favorites",
    });
    const folders = await client.listFolders("tenant_1", "user_1");
    const tag = await client.createTag({ id: "tag_1", tenantId: "tenant_1", name: "cute" });
    const tags = await client.listTags("tenant_1");
    const group = await client.createSubscriptionGroup({
      id: "sub_1",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      title: "Weekly",
      visibility: "private",
    });
    const groups = await client.listSubscriptionGroups("tenant_1", "user_1");

    expect(folder.name).toBe("Favorites");
    expect(folders[0]?.id).toBe("folder_1");
    expect(tag.name).toBe("cute");
    expect(tags[0]?.id).toBe("tag_1");
    expect(group.visibility).toBe("private");
    expect(groups[0]?.title).toBe("Weekly");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/folders", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_web_secret",
      },
      body: JSON.stringify({
        id: "folder_1",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        name: "Favorites",
      }),
    });
  });

  it("manages folder, tag, and subscription group membership links with bearer auth", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (init?.method === "PUT" && url.endsWith("/api/v1/folders/folder_1/packs/pack_1")) {
        return new Response(JSON.stringify({ folderId: "folder_1", packId: "pack_1", sortOrder: 10 }), { status: 200 });
      }
      if (init?.method === "DELETE" && url.endsWith("/api/v1/folders/folder_1/packs/pack_1")) {
        return new Response(null, { status: 204 });
      }
      if (init?.method === "PUT" && url.endsWith("/api/v1/packs/pack_1/tags/tag_1")) {
        return new Response(JSON.stringify({ packId: "pack_1", tagId: "tag_1" }), { status: 200 });
      }
      if (init?.method === "DELETE" && url.endsWith("/api/v1/packs/pack_1/tags/tag_1")) {
        return new Response(null, { status: 204 });
      }
      if (init?.method === "PUT" && url.endsWith("/api/v1/subscription-groups/sub_1/packs/pack_1")) {
        return new Response(JSON.stringify({ subscriptionGroupId: "sub_1", packId: "pack_1", sortOrder: 20 }), { status: 200 });
      }
      if (init?.method === "DELETE" && url.endsWith("/api/v1/subscription-groups/sub_1/packs/pack_1")) {
        return new Response(null, { status: 204 });
      }
      if (url.endsWith("/api/v1/folders/folder_1/packs")) {
        return new Response(JSON.stringify(["pack_1"]), { status: 200 });
      }
      if (url.endsWith("/api/v1/packs/pack_1/tags")) {
        return new Response(JSON.stringify(["tag_1"]), { status: 200 });
      }
      return new Response(JSON.stringify(["pack_1"]), { status: 200 });
    });
    const client = createProductMetadataClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_web_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const folderLink = await client.addPackToFolder({ folderId: "folder_1", packId: "pack_1", sortOrder: 10 });
    const folderPacks = await client.listFolderPacks("folder_1");
    await client.removePackFromFolder("folder_1", "pack_1");
    const packTag = await client.addTagToPack("pack_1", "tag_1");
    const packTags = await client.listPackTags("pack_1");
    await client.removeTagFromPack("pack_1", "tag_1");
    const groupLink = await client.addPackToSubscriptionGroup({
      subscriptionGroupId: "sub_1",
      packId: "pack_1",
      sortOrder: 20,
    });
    const groupPacks = await client.listSubscriptionGroupPacks("sub_1");
    await client.removePackFromSubscriptionGroup("sub_1", "pack_1");

    expect(folderLink.sortOrder).toBe(10);
    expect(folderPacks).toEqual(["pack_1"]);
    expect(packTag.tagId).toBe("tag_1");
    expect(packTags).toEqual(["tag_1"]);
    expect(groupLink.sortOrder).toBe(20);
    expect(groupPacks).toEqual(["pack_1"]);
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/folders/folder_1/packs/pack_1", {
      method: "PUT",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_web_secret",
      },
      body: JSON.stringify({ sortOrder: 10 }),
    });
  });

  it("manages subscription links with bearer auth and one-time secrets", async () => {
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (init?.method === "POST") {
        return new Response(
          JSON.stringify({
            id: "packlink",
            tenantId: "tenant_1",
            ownerUserId: "user_1",
            resourceType: "pack",
            resourceId: "pack_1",
            token: "msm_sub_packlink_secret",
            revokedAt: null,
            createdAt: "2026-05-09T00:00:00Z",
            updatedAt: "2026-05-09T00:00:00Z",
          }),
          { status: 201 },
        );
      }
      if (init?.method === "PATCH") {
        return new Response(
          JSON.stringify({
            id: "packlink",
            tenantId: "tenant_1",
            ownerUserId: "user_1",
            resourceType: "pack",
            resourceId: "pack_1",
            token: "msm_sub_packlink_rotated",
            revokedAt: null,
            createdAt: "2026-05-09T00:00:00Z",
            updatedAt: "2026-05-09T00:00:01Z",
          }),
          { status: 200 },
        );
      }
      if (init?.method === "DELETE") {
        return new Response(null, { status: 204 });
      }
      return new Response(
        JSON.stringify([
          {
            id: "packlink",
            tenantId: "tenant_1",
            ownerUserId: "user_1",
            resourceType: "pack",
            resourceId: "pack_1",
            revokedAt: null,
            createdAt: "2026-05-09T00:00:00Z",
            updatedAt: "2026-05-09T00:00:00Z",
          },
        ]),
        { status: 200 },
      );
    });
    const client = createProductMetadataClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_web_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const created = await client.createSubscriptionLink({
      id: "packlink",
      resourceType: "pack",
      resourceId: "pack_1",
    });
    const links = await client.listSubscriptionLinks("user_1");
    const rotated = await client.rotateSubscriptionLink("packlink");
    await client.revokeSubscriptionLink("packlink");

    expect(created.token).toBe("msm_sub_packlink_secret");
    expect(links[0]).not.toHaveProperty("token");
    expect(rotated.token).toBe("msm_sub_packlink_rotated");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/subscription-access-tokens", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_web_secret",
      },
      body: JSON.stringify({
        id: "packlink",
        resourceType: "pack",
        resourceId: "pack_1",
      }),
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/subscription-access-tokens/packlink/rotate", {
      method: "PATCH",
      headers: {
        Authorization: "Bearer msm_pat_web_secret",
      },
    });
  });
});
