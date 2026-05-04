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
  authToken?: string;
  fetchImpl?: typeof fetch;
}

export interface CreatePersonalAccessTokenRequest {
  id: string;
  userId: string;
  name: string;
  scopes: string[];
  expiresAt: string | null;
}

export interface CreatedPersonalAccessTokenResponse {
  id: string;
  userId: string;
  name: string;
  token: string;
  scopes: string[];
  expiresAt: string | null;
  revokedAt: string | null;
  createdAt: string;
}

export interface PersonalAccessTokenResponse {
  id: string;
  userId: string;
  name: string;
  scopes: string[];
  expiresAt: string | null;
  revokedAt: string | null;
  createdAt: string;
}

export interface PatClient {
  createPersonalAccessToken(
    request: CreatePersonalAccessTokenRequest,
  ): Promise<CreatedPersonalAccessTokenResponse>;
  listPersonalAccessTokens(userId: string): Promise<PersonalAccessTokenResponse[]>;
  revokePersonalAccessToken(tokenId: string): Promise<void>;
}

export interface RegisterLocalUserRequest {
  id: string;
  email: string;
  displayName: string;
  password: string;
}

export interface LocalUserResponse {
  id: string;
  email: string;
  displayName: string;
}

export interface LoginLocalUserRequest {
  email: string;
  password: string;
  tokenId: string;
  tokenName: string;
  scopes: string[];
  expiresAt: string | null;
}

export interface LocalAuthClient {
  registerLocalUser(request: RegisterLocalUserRequest): Promise<LocalUserResponse>;
  loginLocalUser(request: LoginLocalUserRequest): Promise<CreatedPersonalAccessTokenResponse>;
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
      const response = await fetchOptional(fetchImpl, packListUrl(baseUrl, userId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list sticker packs: HTTP ${response.status}`);
      }

      const records = (await response.json()) as ApiStickerPackRecord[];
      return records.map(mapApiPackRecord);
    },
  };
}

export function createPatClient(options: PackClientOptions = {}): PatClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    throw new Error("PAT API client requires a base URL");
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async createPersonalAccessToken(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/pats`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to create PAT: HTTP ${response.status}`);
      }

      return (await response.json()) as CreatedPersonalAccessTokenResponse;
    },
    async listPersonalAccessTokens(userId) {
      const response = await fetchOptional(fetchImpl, patListUrl(baseUrl, userId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list PATs: HTTP ${response.status}`);
      }

      return (await response.json()) as PersonalAccessTokenResponse[];
    },
    async revokePersonalAccessToken(tokenId) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/pats/${encodeURIComponent(tokenId)}`, {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to revoke PAT: HTTP ${response.status}`);
      }
    },
  };
}

export function createLocalAuthClient(options: PackClientOptions = {}): LocalAuthClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    throw new Error("Local auth API client requires a base URL");
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async registerLocalUser(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/auth/local/register`, {
        method: "POST",
        headers: jsonHeaders(undefined),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to register local user: HTTP ${response.status}`);
      }

      return (await response.json()) as LocalUserResponse;
    },
    async loginLocalUser(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/auth/local/login`, {
        method: "POST",
        headers: jsonHeaders(undefined),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to login local user: HTTP ${response.status}`);
      }

      return (await response.json()) as CreatedPersonalAccessTokenResponse;
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
  const path = `${trimBaseUrl(baseUrl)}/api/v1/packs`;
  const query = new URLSearchParams({ userId });
  return `${path}?${query.toString()}`;
}

export function patListUrl(baseUrl: string, userId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/pats`;
  const query = new URLSearchParams({ userId });
  return `${path}?${query.toString()}`;
}

function trimBaseUrl(baseUrl: string) {
  return baseUrl.trim().replace(/\/+$/, "");
}

function fetchOptional(fetchImpl: typeof fetch, url: string, init: RequestInit | undefined) {
  return init ? fetchImpl(url, init) : fetchImpl(url);
}

function authInit(authToken: string | undefined): RequestInit | undefined {
  const token = authToken?.trim();
  if (!token) {
    return undefined;
  }

  return {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  };
}

function jsonHeaders(authToken: string | undefined): Record<string, string> {
  return {
    "Content-Type": "application/json",
    ...(authInit(authToken)?.headers as Record<string, string> | undefined),
  };
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
