import { describe, expect, it, vi } from "vitest";

import {
  createPackClient,
  createLocalAuthClient,
  createPatClient,
  mapApiPackRecord,
  packListUrl,
  type ApiStickerPackRecord,
} from "./api-client";

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
