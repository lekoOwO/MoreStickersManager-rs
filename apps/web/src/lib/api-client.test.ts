import { describe, expect, it, vi } from "vitest";

import { createPackClient, mapApiPackRecord, packListUrl, type ApiStickerPackRecord } from "./api-client";

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
