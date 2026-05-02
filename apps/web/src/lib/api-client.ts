import { listMockStickerPacks, type PackVisibility, type StickerPackSummary } from "./sticker-packs";

export interface ApiStickerPack {
  id: string;
  title: string;
  stickers?: unknown[];
}

export interface ApiStickerPackRecord {
  id: string;
  title?: string;
  visibility?: string;
  compatibility_id?: string;
  compatibilityId?: string;
  source_provider?: string | null;
  sourceProvider?: string | null;
  sticker_pack?: ApiStickerPack;
  stickerPack?: ApiStickerPack;
  updated_at?: string;
  updatedAt?: string;
}

export interface PackClient {
  listStickerPacks(): Promise<StickerPackSummary[]>;
}

export interface PackClientOptions {
  baseUrl?: string;
  userId?: string;
  fetchImpl?: typeof fetch;
}

export function createPackClient(options: PackClientOptions = {}): PackClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    return {
      listStickerPacks: listMockStickerPacks,
    };
  }

  const userId = options.userId?.trim() || "demo";
  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async listStickerPacks() {
      const response = await fetchImpl(packListUrl(baseUrl, userId));
      if (!response.ok) {
        throw new Error(`Failed to list sticker packs: HTTP ${response.status}`);
      }

      const records = (await response.json()) as ApiStickerPackRecord[];
      return records.map(mapApiPackRecord);
    },
  };
}

export function mapApiPackRecord(record: ApiStickerPackRecord): StickerPackSummary {
  const stickerPack = record.sticker_pack ?? record.stickerPack;
  const compatibilityId = record.compatibility_id ?? record.compatibilityId ?? stickerPack?.id ?? record.id;
  const sourceProvider = record.source_provider ?? record.sourceProvider;

  return {
    id: record.id,
    title: record.title ?? stickerPack?.title ?? record.id,
    provider: inferProvider(sourceProvider, compatibilityId),
    visibility: mapVisibility(record.visibility),
    stickerCount: stickerPack?.stickers?.length ?? 0,
    subscriptionReady: true,
    updatedAt: mapUpdatedDate(record.updated_at ?? record.updatedAt),
  };
}

export function packListUrl(baseUrl: string, userId: string) {
  const trimmedBase = baseUrl.trim().replace(/\/+$/, "");
  const path = `${trimmedBase}/api/v1/packs`;
  const query = new URLSearchParams({ userId });
  return `${path}?${query.toString()}`;
}

function inferProvider(
  sourceProvider: string | null | undefined,
  compatibilityId: string,
): StickerPackSummary["provider"] {
  const provider = sourceProvider?.toLowerCase();
  const id = compatibilityId.toLowerCase();

  if (provider?.includes("telegram") || id.startsWith("morestickers:telegram:")) {
    return "Telegram";
  }

  if (provider?.includes("emoji") || id.startsWith("morestickers:line:emoji-pack:")) {
    return "LINE Emojis";
  }

  return "LINE Stickers";
}

function mapVisibility(visibility: string | undefined): PackVisibility {
  return visibility?.toLowerCase() === "public" ? "public" : "private";
}

function mapUpdatedDate(updatedAt: string | undefined) {
  return updatedAt?.split("T")[0] ?? "unknown";
}
