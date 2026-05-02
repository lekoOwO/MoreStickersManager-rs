export type PackVisibility = "public" | "private" | "member";

export interface StickerPackSummary {
  id: string;
  title: string;
  provider: "Telegram" | "LINE Stickers" | "LINE Emojis";
  visibility: PackVisibility;
  stickerCount: number;
  subscriptionReady: boolean;
  updatedAt: string;
}

export const mockStickerPacks: StickerPackSummary[] = [
  {
    id: "telegram-cats",
    title: "Telegram Cats",
    provider: "Telegram",
    visibility: "public",
    stickerCount: 48,
    subscriptionReady: true,
    updatedAt: "2026-05-03",
  },
  {
    id: "line-cafe",
    title: "LINE Cafe Crew",
    provider: "LINE Stickers",
    visibility: "member",
    stickerCount: 32,
    subscriptionReady: true,
    updatedAt: "2026-05-02",
  },
  {
    id: "line-emoji-cats",
    title: "LINE Emoji Cats",
    provider: "LINE Emojis",
    visibility: "private",
    stickerCount: 40,
    subscriptionReady: false,
    updatedAt: "2026-05-01",
  },
];

export async function listMockStickerPacks(): Promise<StickerPackSummary[]> {
  return [...mockStickerPacks];
}
