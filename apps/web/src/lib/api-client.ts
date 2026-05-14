import { listMockStickerPacks, type PackVisibility, type StickerPackSummary } from "./sticker-packs";

export type WritablePackVisibility = Extract<PackVisibility, "public" | "private">;

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
  importStickerPack(request: ImportStickerPackRequest): Promise<void>;
  updateStickerPack(request: UpdateStickerPackRequest): Promise<void>;
  deleteStickerPack(packId: string): Promise<void>;
}

export type ProviderImportSource = "telegram" | "line-stickers";

export interface CreateProviderImportPlanRequest {
  tenantId: string;
  ownerUserId: string;
  providerId: ProviderImportSource;
  remoteId: string;
  baseUrl: string | null;
}

export interface CreateProviderImportJobRequest extends CreateProviderImportPlanRequest {
  id: string;
  targetPackId: string | null;
}

export interface ProviderHttpHeader {
  name: string;
  value: string;
}

export interface ProviderHttpRequestPlan {
  method: string;
  url: string;
  redactedHeaders: ProviderHttpHeader[];
}

export interface ProviderImportPlan {
  providerId: ProviderImportSource;
  remoteId: string;
  metadataRequest: ProviderHttpRequestPlan;
  assetStrategy: "telegramBotFileApi" | "directRemoteUrls";
}

export interface ProviderImportJob {
  id: string;
  tenantId: string;
  ownerUserId: string;
  providerId: ProviderImportSource;
  remoteId: string;
  targetPackId: string | null;
  status: string;
  request: unknown;
  result: unknown | null;
  errorSummary: string | null;
  attemptCount: number;
  maxAttempts: number;
  nextAttemptAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProviderImportJobEvent {
  jobId: string;
  sequence: number;
  level: string;
  stage: string;
  message: string;
  metadata: unknown;
  createdAt: string;
}

export interface ProviderConfigResponse {
  id: string;
  tenantId: string;
  providerId: ProviderImportSource;
  name: string;
  config: Record<string, unknown>;
  isEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertProviderConfigRequest {
  tenantId: string;
  providerId: ProviderImportSource;
  name: string;
  config: Record<string, unknown>;
  isEnabled: boolean;
}

export interface ProviderImportClient {
  createProviderImportPlan(request: CreateProviderImportPlanRequest): Promise<ProviderImportPlan>;
  createProviderImportJob(request: CreateProviderImportJobRequest): Promise<ProviderImportJob>;
  getProviderImportJob(jobId: string): Promise<ProviderImportJob>;
  listProviderImportJobEvents(jobId: string): Promise<ProviderImportJobEvent[]>;
  listProviderConfigs(tenantId: string): Promise<ProviderConfigResponse[]>;
  upsertProviderConfig(configId: string, request: UpsertProviderConfigRequest): Promise<ProviderConfigResponse>;
  deleteProviderConfig(configId: string): Promise<void>;
}

export interface PortabilityClient {
  exportUserData(userId: string): Promise<unknown>;
  importUserData(request: { tenantId: string; export: unknown }): Promise<void>;
}

export interface ImportStickerPackRequest {
  tenantId: string;
  ownerUserId: string;
  packId: string;
  visibility: WritablePackVisibility;
  pack: unknown;
}

export interface UpdateStickerPackRequest {
  packId: string;
  title: string;
  visibility: WritablePackVisibility;
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

export interface PatScopePolicyResponse {
  userId: string;
  allowedScopes: string[];
}

export interface PatClient {
  createPersonalAccessToken(
    request: CreatePersonalAccessTokenRequest,
  ): Promise<CreatedPersonalAccessTokenResponse>;
  listPersonalAccessTokens(userId: string): Promise<PersonalAccessTokenResponse[]>;
  getPatScopePolicy(userId: string): Promise<PatScopePolicyResponse>;
  revokePersonalAccessToken(tokenId: string): Promise<void>;
}

export interface RegisterLocalUserRequest {
  id: string;
  email: string;
  displayName: string;
  password: string;
  tenantId: string;
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

export interface StartOidcLoginRequest {
  tenantId: string;
  providerId: string;
  redirectUri: string;
}

export interface OidcLoginStartResponse {
  tenantId: string;
  providerId: string;
  authorizationUrl: string;
  state: string;
  nonce: string;
  expiresAt: string;
}

export interface CompleteOidcLoginRequest {
  state: string;
  nonce: string;
  authorizationCode: string | null;
  issuer: string;
  audience: string;
  providerSubject: string;
  email: string;
  displayName: string;
  tokenId: string;
  tokenName: string;
  scopes: string[];
  expiresAt: string | null;
}

export interface OidcAuthClient {
  startOidcLogin(request: StartOidcLoginRequest): Promise<OidcLoginStartResponse>;
  completeOidcLogin(request: CompleteOidcLoginRequest): Promise<CreatedPersonalAccessTokenResponse>;
}

export type TenantMemberRole = "admin" | "user";

export interface TenantMemberResponse {
  tenantId: string;
  userId: string;
  role: TenantMemberRole;
  createdAt: string;
}

export interface TenantSettingsResponse {
  tenantId: string;
  name: string;
  publicAssetUrl: string | null;
  localRegistrationEnabled: boolean;
  createdAt: string;
}

export interface UpdateTenantSettingsRequest {
  name: string;
  publicAssetUrl: string | null;
  localRegistrationEnabled: boolean;
}

export interface TenantUserResponse {
  id: string;
  email: string;
  displayName: string;
  isDisabled: boolean;
  createdAt: string;
}

export interface TenantRoleResponse {
  id: string;
  tenantId: string | null;
  name: string;
  permissions: string[];
  createdAt: string;
}

export interface UpsertTenantRoleRequest {
  name: string;
  permissions: string[];
}

export interface OidcProviderResponse {
  id: string;
  tenantId: string;
  displayName: string;
  issuerUrl: string;
  clientId: string;
  clientSecret: string;
  scopes: string[];
  isEnabled: boolean;
  allowRegistration: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertOidcProviderRequest {
  displayName: string;
  issuerUrl: string;
  clientId: string;
  clientSecret: string;
  scopes: string[];
  isEnabled: boolean;
  allowRegistration: boolean;
}

export interface TenantAdminClient {
  listTenantMembers(tenantId: string): Promise<TenantMemberResponse[]>;
  setTenantMemberRole(
    tenantId: string,
    userId: string,
    role: TenantMemberRole,
  ): Promise<TenantMemberResponse>;
  getTenantSettings(tenantId: string): Promise<TenantSettingsResponse>;
  updateTenantSettings(
    tenantId: string,
    request: UpdateTenantSettingsRequest,
  ): Promise<TenantSettingsResponse>;
  setTenantUserStatus(
    tenantId: string,
    userId: string,
    isDisabled: boolean,
  ): Promise<TenantUserResponse>;
  listTenantRoles(tenantId: string): Promise<TenantRoleResponse[]>;
  upsertTenantRole(
    tenantId: string,
    roleId: string,
    request: UpsertTenantRoleRequest,
  ): Promise<TenantRoleResponse>;
  listOidcProviders(tenantId: string): Promise<OidcProviderResponse[]>;
  upsertOidcProvider(
    tenantId: string,
    providerId: string,
    request: UpsertOidcProviderRequest,
  ): Promise<OidcProviderResponse>;
  deleteOidcProvider(tenantId: string, providerId: string): Promise<void>;
}

export interface ProductMetadataFolder {
  id: string;
  tenantId: string;
  ownerUserId: string;
  name: string;
  createdAt: string;
}

export interface ProductMetadataTag {
  id: string;
  tenantId: string;
  name: string;
  createdAt: string;
}

export interface ProductMetadataSubscriptionGroup {
  id: string;
  tenantId: string;
  ownerUserId: string;
  title: string;
  visibility: WritablePackVisibility;
  createdAt: string;
}

export type CreateFolderRequest = Omit<ProductMetadataFolder, "createdAt">;
export type CreateTagRequest = Omit<ProductMetadataTag, "createdAt">;
export type CreateSubscriptionGroupRequest = Omit<ProductMetadataSubscriptionGroup, "createdAt">;

export interface FolderPackLink {
  folderId: string;
  packId: string;
  sortOrder: number;
}

export interface PackTagLink {
  packId: string;
  tagId: string;
}

export interface SubscriptionGroupPackLink {
  subscriptionGroupId: string;
  packId: string;
  sortOrder: number;
}

export type SubscriptionAccessResourceType = "pack" | "subscriptionGroup";

export interface CreateSubscriptionAccessTokenRequest {
  id: string;
  resourceType: SubscriptionAccessResourceType;
  resourceId: string;
}

export interface SubscriptionAccessTokenResponse {
  id: string;
  tenantId: string;
  ownerUserId: string;
  resourceType: SubscriptionAccessResourceType;
  resourceId: string;
  revokedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreatedSubscriptionAccessTokenResponse extends SubscriptionAccessTokenResponse {
  token: string;
}

export interface ProductMetadataClient {
  listFolders(tenantId: string, ownerUserId: string): Promise<ProductMetadataFolder[]>;
  createFolder(request: CreateFolderRequest): Promise<ProductMetadataFolder>;
  listFolderPacks(folderId: string): Promise<string[]>;
  addPackToFolder(request: FolderPackLink): Promise<FolderPackLink>;
  removePackFromFolder(folderId: string, packId: string): Promise<void>;
  listTags(tenantId: string): Promise<ProductMetadataTag[]>;
  createTag(request: CreateTagRequest): Promise<ProductMetadataTag>;
  listPackTags(packId: string): Promise<string[]>;
  addTagToPack(packId: string, tagId: string): Promise<PackTagLink>;
  removeTagFromPack(packId: string, tagId: string): Promise<void>;
  listSubscriptionGroups(tenantId: string, ownerUserId: string): Promise<ProductMetadataSubscriptionGroup[]>;
  createSubscriptionGroup(request: CreateSubscriptionGroupRequest): Promise<ProductMetadataSubscriptionGroup>;
  listSubscriptionGroupPacks(subscriptionGroupId: string): Promise<string[]>;
  addPackToSubscriptionGroup(request: SubscriptionGroupPackLink): Promise<SubscriptionGroupPackLink>;
  removePackFromSubscriptionGroup(subscriptionGroupId: string, packId: string): Promise<void>;
  listSubscriptionLinks(userId: string): Promise<SubscriptionAccessTokenResponse[]>;
  createSubscriptionLink(
    request: CreateSubscriptionAccessTokenRequest,
  ): Promise<CreatedSubscriptionAccessTokenResponse>;
  rotateSubscriptionLink(tokenId: string): Promise<CreatedSubscriptionAccessTokenResponse>;
  revokeSubscriptionLink(tokenId: string): Promise<void>;
}

export function createPackClient(options: PackClientOptions = {}): PackClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    return {
      listStickerPacks: listMockStickerPacks,
      async importStickerPack() {
        throw new Error("Pack import requires VITE_MSM_API_BASE_URL");
      },
      async updateStickerPack() {
        throw new Error("Pack update requires VITE_MSM_API_BASE_URL");
      },
      async deleteStickerPack() {
        throw new Error("Pack delete requires VITE_MSM_API_BASE_URL");
      },
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
    async importStickerPack(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/packs/import`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to import sticker pack: HTTP ${response.status}`);
      }
    },
    async updateStickerPack(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/packs/${encodeURIComponent(request.packId)}`, {
        method: "PATCH",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify({
          title: request.title,
          visibility: request.visibility,
        }),
      });
      if (!response.ok) {
        throw new Error(`Failed to update sticker pack: HTTP ${response.status}`);
      }
    },
    async deleteStickerPack(packId) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/packs/${encodeURIComponent(packId)}`, {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to delete sticker pack: HTTP ${response.status}`);
      }
    },
  };
}

export function createProviderImportClient(options: PackClientOptions = {}): ProviderImportClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    return {
      async createProviderImportPlan() {
        throw new Error("Provider import planning requires VITE_MSM_API_BASE_URL");
      },
      async createProviderImportJob() {
        throw new Error("Provider import jobs require VITE_MSM_API_BASE_URL");
      },
      async getProviderImportJob() {
        throw new Error("Provider import jobs require VITE_MSM_API_BASE_URL");
      },
      async listProviderImportJobEvents() {
        throw new Error("Provider import job events require VITE_MSM_API_BASE_URL");
      },
      async listProviderConfigs() {
        throw new Error("Provider configs require VITE_MSM_API_BASE_URL");
      },
      async upsertProviderConfig() {
        throw new Error("Provider configs require VITE_MSM_API_BASE_URL");
      },
      async deleteProviderConfig() {
        throw new Error("Provider configs require VITE_MSM_API_BASE_URL");
      },
    };
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async createProviderImportPlan(request) {
      const response = await fetchImpl(providerImportPlanUrl(baseUrl), {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify({
          ...request,
          baseUrl: request.baseUrl?.trim() || null,
        }),
      });
      if (!response.ok) {
        throw new Error(`Failed to create provider import plan: HTTP ${response.status}`);
      }

      return (await response.json()) as ProviderImportPlan;
    },
    async createProviderImportJob(request) {
      const response = await fetchImpl(providerImportJobsUrl(baseUrl), {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify({
          ...request,
          targetPackId: request.targetPackId?.trim() || null,
          baseUrl: request.baseUrl?.trim() || null,
        }),
      });
      if (!response.ok) {
        throw new Error(`Failed to create provider import job: HTTP ${response.status}`);
      }

      return (await response.json()) as ProviderImportJob;
    },
    async getProviderImportJob(jobId) {
      const response = await fetchImpl(providerImportJobUrl(baseUrl, jobId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to get provider import job: HTTP ${response.status}`);
      }

      return (await response.json()) as ProviderImportJob;
    },
    async listProviderImportJobEvents(jobId) {
      const response = await fetchImpl(providerImportJobEventsUrl(baseUrl, jobId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list provider import job events: HTTP ${response.status}`);
      }

      return (await response.json()) as ProviderImportJobEvent[];
    },
    async listProviderConfigs(tenantId) {
      const response = await fetchImpl(providerConfigsUrl(baseUrl, tenantId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list provider configs: HTTP ${response.status}`);
      }

      return (await response.json()) as ProviderConfigResponse[];
    },
    async upsertProviderConfig(configId, request) {
      const response = await fetchImpl(providerConfigUrl(baseUrl, configId), {
        method: "PUT",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to upsert provider config: HTTP ${response.status}`);
      }

      return (await response.json()) as ProviderConfigResponse;
    },
    async deleteProviderConfig(configId) {
      const response = await fetchImpl(providerConfigUrl(baseUrl, configId), {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to delete provider config: HTTP ${response.status}`);
      }
    },
  };
}

export function createPortabilityClient(options: PackClientOptions = {}): PortabilityClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    return {
      async exportUserData() {
        throw new Error("Portable user export requires VITE_MSM_API_BASE_URL");
      },
      async importUserData() {
        throw new Error("Portable user import requires VITE_MSM_API_BASE_URL");
      },
    };
  }

  const fetchImpl = options.fetchImpl ?? fetch;
  return {
    async exportUserData(userId) {
      const response = await fetchOptional(fetchImpl, portableUserExportUrl(baseUrl, userId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to export portable user data: HTTP ${response.status}`);
      }
      return response.json();
    },
    async importUserData(request) {
      const response = await fetchImpl(portableUserImportUrl(baseUrl), {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to import portable user data: HTTP ${response.status}`);
      }
    },
  };
}

export function createProductMetadataClient(options: PackClientOptions = {}): ProductMetadataClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    return mockProductMetadataClient();
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async listFolders(tenantId, ownerUserId) {
      const response = await fetchOptional(fetchImpl, folderListUrl(baseUrl, tenantId, ownerUserId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list folders: HTTP ${response.status}`);
      }

      return (await response.json()) as ProductMetadataFolder[];
    },
    async createFolder(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/folders`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to create folder: HTTP ${response.status}`);
      }

      return (await response.json()) as ProductMetadataFolder;
    },
    async listFolderPacks(folderId) {
      const response = await fetchOptional(fetchImpl, folderPackListUrl(baseUrl, folderId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list folder packs: HTTP ${response.status}`);
      }

      return (await response.json()) as string[];
    },
    async addPackToFolder(request) {
      const response = await fetchImpl(
        `${folderPackListUrl(baseUrl, request.folderId)}/${encodeURIComponent(request.packId)}`,
        {
          method: "PUT",
          headers: jsonHeaders(options.authToken),
          body: JSON.stringify({ sortOrder: request.sortOrder }),
        },
      );
      if (!response.ok) {
        throw new Error(`Failed to add pack to folder: HTTP ${response.status}`);
      }

      return (await response.json()) as FolderPackLink;
    },
    async removePackFromFolder(folderId, packId) {
      const response = await fetchImpl(`${folderPackListUrl(baseUrl, folderId)}/${encodeURIComponent(packId)}`, {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to remove pack from folder: HTTP ${response.status}`);
      }
    },
    async listTags(tenantId) {
      const response = await fetchOptional(fetchImpl, tagListUrl(baseUrl, tenantId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list tags: HTTP ${response.status}`);
      }

      return (await response.json()) as ProductMetadataTag[];
    },
    async createTag(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/tags`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to create tag: HTTP ${response.status}`);
      }

      return (await response.json()) as ProductMetadataTag;
    },
    async listPackTags(packId) {
      const response = await fetchOptional(fetchImpl, packTagListUrl(baseUrl, packId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list pack tags: HTTP ${response.status}`);
      }

      return (await response.json()) as string[];
    },
    async addTagToPack(packId, tagId) {
      const response = await fetchImpl(`${packTagListUrl(baseUrl, packId)}/${encodeURIComponent(tagId)}`, {
        method: "PUT",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to add tag to pack: HTTP ${response.status}`);
      }

      return (await response.json()) as PackTagLink;
    },
    async removeTagFromPack(packId, tagId) {
      const response = await fetchImpl(`${packTagListUrl(baseUrl, packId)}/${encodeURIComponent(tagId)}`, {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to remove tag from pack: HTTP ${response.status}`);
      }
    },
    async listSubscriptionGroups(tenantId, ownerUserId) {
      const response = await fetchOptional(
        fetchImpl,
        subscriptionGroupListUrl(baseUrl, tenantId, ownerUserId),
        authInit(options.authToken),
      );
      if (!response.ok) {
        throw new Error(`Failed to list subscription groups: HTTP ${response.status}`);
      }

      return (await response.json()) as ProductMetadataSubscriptionGroup[];
    },
    async createSubscriptionGroup(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/subscription-groups`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to create subscription group: HTTP ${response.status}`);
      }

      return (await response.json()) as ProductMetadataSubscriptionGroup;
    },
    async listSubscriptionGroupPacks(subscriptionGroupId) {
      const response = await fetchOptional(
        fetchImpl,
        subscriptionGroupPackListUrl(baseUrl, subscriptionGroupId),
        authInit(options.authToken),
      );
      if (!response.ok) {
        throw new Error(`Failed to list subscription group packs: HTTP ${response.status}`);
      }

      return (await response.json()) as string[];
    },
    async addPackToSubscriptionGroup(request) {
      const response = await fetchImpl(
        `${subscriptionGroupPackListUrl(baseUrl, request.subscriptionGroupId)}/${encodeURIComponent(request.packId)}`,
        {
          method: "PUT",
          headers: jsonHeaders(options.authToken),
          body: JSON.stringify({ sortOrder: request.sortOrder }),
        },
      );
      if (!response.ok) {
        throw new Error(`Failed to add pack to subscription group: HTTP ${response.status}`);
      }

      return (await response.json()) as SubscriptionGroupPackLink;
    },
    async removePackFromSubscriptionGroup(subscriptionGroupId, packId) {
      const response = await fetchImpl(
        `${subscriptionGroupPackListUrl(baseUrl, subscriptionGroupId)}/${encodeURIComponent(packId)}`,
        {
          method: "DELETE",
          ...authInit(options.authToken),
        },
      );
      if (!response.ok) {
        throw new Error(`Failed to remove pack from subscription group: HTTP ${response.status}`);
      }
    },
    async listSubscriptionLinks(userId) {
      const response = await fetchOptional(fetchImpl, subscriptionLinkListUrl(baseUrl, userId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list subscription links: HTTP ${response.status}`);
      }

      return (await response.json()) as SubscriptionAccessTokenResponse[];
    },
    async createSubscriptionLink(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/subscription-access-tokens`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to create subscription link: HTTP ${response.status}`);
      }

      return (await response.json()) as CreatedSubscriptionAccessTokenResponse;
    },
    async rotateSubscriptionLink(tokenId) {
      const response = await fetchImpl(
        `${trimBaseUrl(baseUrl)}/api/v1/subscription-access-tokens/${encodeURIComponent(tokenId)}/rotate`,
        {
          method: "PATCH",
          ...authInit(options.authToken),
        },
      );
      if (!response.ok) {
        throw new Error(`Failed to rotate subscription link: HTTP ${response.status}`);
      }

      return (await response.json()) as CreatedSubscriptionAccessTokenResponse;
    },
    async revokeSubscriptionLink(tokenId) {
      const response = await fetchImpl(
        `${trimBaseUrl(baseUrl)}/api/v1/subscription-access-tokens/${encodeURIComponent(tokenId)}`,
        {
          method: "DELETE",
          ...authInit(options.authToken),
        },
      );
      if (!response.ok) {
        throw new Error(`Failed to revoke subscription link: HTTP ${response.status}`);
      }
    },
  };
}

export function createTenantAdminClient(options: PackClientOptions = {}): TenantAdminClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    return mockTenantAdminClient();
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async listTenantMembers(tenantId) {
      const response = await fetchOptional(fetchImpl, tenantMemberListUrl(baseUrl, tenantId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list tenant members: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantMemberResponse[];
    },
    async setTenantMemberRole(tenantId, userId, role) {
      const response = await fetchImpl(tenantMemberUrl(baseUrl, tenantId, userId), {
        method: "PUT",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify({ role }),
      });
      if (!response.ok) {
        throw new Error(`Failed to set tenant member role: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantMemberResponse;
    },
    async getTenantSettings(tenantId) {
      const response = await fetchOptional(fetchImpl, tenantSettingsUrl(baseUrl, tenantId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to read tenant settings: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantSettingsResponse;
    },
    async updateTenantSettings(tenantId, request) {
      const response = await fetchImpl(tenantSettingsUrl(baseUrl, tenantId), {
        method: "PUT",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to update tenant settings: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantSettingsResponse;
    },
    async setTenantUserStatus(tenantId, userId, isDisabled) {
      const response = await fetchImpl(tenantUserStatusUrl(baseUrl, tenantId, userId), {
        method: "PUT",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify({ isDisabled }),
      });
      if (!response.ok) {
        throw new Error(`Failed to update tenant user status: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantUserResponse;
    },
    async listTenantRoles(tenantId) {
      const response = await fetchOptional(fetchImpl, tenantRolesUrl(baseUrl, tenantId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list tenant roles: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantRoleResponse[];
    },
    async upsertTenantRole(tenantId, roleId, request) {
      const response = await fetchImpl(tenantRoleUrl(baseUrl, tenantId, roleId), {
        method: "PUT",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to upsert tenant role: HTTP ${response.status}`);
      }

      return (await response.json()) as TenantRoleResponse;
    },
    async listOidcProviders(tenantId) {
      const response = await fetchOptional(fetchImpl, tenantOidcProvidersUrl(baseUrl, tenantId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to list OIDC providers: HTTP ${response.status}`);
      }

      return (await response.json()) as OidcProviderResponse[];
    },
    async upsertOidcProvider(tenantId, providerId, request) {
      const response = await fetchImpl(tenantOidcProviderUrl(baseUrl, tenantId, providerId), {
        method: "PUT",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to upsert OIDC provider: HTTP ${response.status}`);
      }

      return (await response.json()) as OidcProviderResponse;
    },
    async deleteOidcProvider(tenantId, providerId) {
      const response = await fetchImpl(tenantOidcProviderUrl(baseUrl, tenantId, providerId), {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      if (!response.ok) {
        throw new Error(`Failed to delete OIDC provider: HTTP ${response.status}`);
      }
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
    async getPatScopePolicy(userId) {
      const response = await fetchOptional(fetchImpl, patScopePolicyUrl(baseUrl, userId), authInit(options.authToken));
      if (!response.ok) {
        throw new Error(`Failed to load PAT scope policy: HTTP ${response.status}`);
      }

      return (await response.json()) as PatScopePolicyResponse;
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

export function createOidcAuthClient(options: PackClientOptions = {}): OidcAuthClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    throw new Error("OIDC auth API client requires a base URL");
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async startOidcLogin(request) {
      const response = await fetchImpl(
        oidcLoginStartUrl(baseUrl, request.tenantId, request.providerId, request.redirectUri),
      );
      if (!response.ok) {
        throw new Error(`Failed to start OIDC login: HTTP ${response.status}`);
      }

      return (await response.json()) as OidcLoginStartResponse;
    },
    async completeOidcLogin(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/auth/oidc/callback`, {
        method: "POST",
        headers: jsonHeaders(undefined),
        body: JSON.stringify(request),
      });
      if (!response.ok) {
        throw new Error(`Failed to complete OIDC login: HTTP ${response.status}`);
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

export function patScopePolicyUrl(baseUrl: string, userId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/pats/scope-policy`;
  const query = new URLSearchParams({ userId });
  return `${path}?${query.toString()}`;
}

export function providerConfigsUrl(baseUrl: string, tenantId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/provider-configs`;
  const query = new URLSearchParams({ tenantId });
  return `${path}?${query.toString()}`;
}

export function providerConfigUrl(baseUrl: string, configId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/provider-configs/${encodeURIComponent(configId)}`;
}

export function providerImportPlanUrl(baseUrl: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/provider-imports/plan`;
}

export function providerImportJobsUrl(baseUrl: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/provider-import-jobs`;
}

export function providerImportJobUrl(baseUrl: string, jobId: string) {
  return `${providerImportJobsUrl(baseUrl)}/${encodeURIComponent(jobId)}`;
}

export function providerImportJobEventsUrl(baseUrl: string, jobId: string) {
  return `${providerImportJobUrl(baseUrl, jobId)}/events`;
}

export function portableUserExportUrl(baseUrl: string, userId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/portable/user-export`;
  const query = new URLSearchParams({ userId });
  return `${path}?${query.toString()}`;
}

export function portableUserImportUrl(baseUrl: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/portable/user-import`;
}

export function oidcLoginStartUrl(baseUrl: string, tenantId: string, providerId: string, redirectUri: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/auth/oidc/${encodeURIComponent(tenantId)}/${encodeURIComponent(providerId)}/login`;
  const query = new URLSearchParams({ redirectUri });
  return `${path}?${query.toString()}`;
}

export function folderListUrl(baseUrl: string, tenantId: string, ownerUserId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/folders`;
  const query = new URLSearchParams({ tenantId, ownerUserId });
  return `${path}?${query.toString()}`;
}

export function tagListUrl(baseUrl: string, tenantId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/tags`;
  const query = new URLSearchParams({ tenantId });
  return `${path}?${query.toString()}`;
}

export function subscriptionGroupListUrl(baseUrl: string, tenantId: string, ownerUserId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/subscription-groups`;
  const query = new URLSearchParams({ tenantId, ownerUserId });
  return `${path}?${query.toString()}`;
}

export function subscriptionLinkListUrl(baseUrl: string, userId: string) {
  const path = `${trimBaseUrl(baseUrl)}/api/v1/subscription-access-tokens`;
  const query = new URLSearchParams({ userId });
  return `${path}?${query.toString()}`;
}

export function folderPackListUrl(baseUrl: string, folderId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/folders/${encodeURIComponent(folderId)}/packs`;
}

export function packTagListUrl(baseUrl: string, packId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/packs/${encodeURIComponent(packId)}/tags`;
}

export function subscriptionGroupPackListUrl(baseUrl: string, subscriptionGroupId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/subscription-groups/${encodeURIComponent(subscriptionGroupId)}/packs`;
}

export function tenantMemberListUrl(baseUrl: string, tenantId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/tenants/${encodeURIComponent(tenantId)}/members`;
}

export function tenantMemberUrl(baseUrl: string, tenantId: string, userId: string) {
  return `${tenantMemberListUrl(baseUrl, tenantId)}/${encodeURIComponent(userId)}`;
}

export function tenantSettingsUrl(baseUrl: string, tenantId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/tenants/${encodeURIComponent(tenantId)}/settings`;
}

export function tenantUserStatusUrl(baseUrl: string, tenantId: string, userId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/tenants/${encodeURIComponent(tenantId)}/users/${encodeURIComponent(userId)}/status`;
}

export function tenantRolesUrl(baseUrl: string, tenantId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/tenants/${encodeURIComponent(tenantId)}/roles`;
}

export function tenantRoleUrl(baseUrl: string, tenantId: string, roleId: string) {
  return `${tenantRolesUrl(baseUrl, tenantId)}/${encodeURIComponent(roleId)}`;
}

export function tenantOidcProvidersUrl(baseUrl: string, tenantId: string) {
  return `${trimBaseUrl(baseUrl)}/api/v1/tenants/${encodeURIComponent(tenantId)}/oidc-providers`;
}

export function tenantOidcProviderUrl(baseUrl: string, tenantId: string, providerId: string) {
  return `${tenantOidcProvidersUrl(baseUrl, tenantId)}/${encodeURIComponent(providerId)}`;
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

function mockProductMetadataClient(): ProductMetadataClient {
  const folders: ProductMetadataFolder[] = [
    {
      id: "folder_favorites",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      name: "Favorites",
      createdAt: "2026-05-09T00:00:00Z",
    },
  ];
  const tags: ProductMetadataTag[] = [
    {
      id: "tag_reaction",
      tenantId: "tenant_1",
      name: "reaction",
      createdAt: "2026-05-09T00:00:00Z",
    },
  ];
  const groups: ProductMetadataSubscriptionGroup[] = [
    {
      id: "sub_weekly",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      title: "Weekly sync",
      visibility: "private",
      createdAt: "2026-05-09T00:00:00Z",
    },
  ];
  const subscriptionLinks: SubscriptionAccessTokenResponse[] = [
    {
      id: "packlink",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      resourceType: "pack",
      resourceId: "pack_demo",
      revokedAt: null,
      createdAt: "2026-05-09T00:00:00Z",
      updatedAt: "2026-05-09T00:00:00Z",
    },
  ];

  return {
    async listFolders() {
      return [...folders];
    },
    async createFolder(request) {
      throw new Error(`Folder creation requires VITE_MSM_API_BASE_URL: ${request.id}`);
    },
    async listFolderPacks() {
      return [];
    },
    async addPackToFolder(request) {
      throw new Error(`Folder membership requires VITE_MSM_API_BASE_URL: ${request.folderId}`);
    },
    async removePackFromFolder(folderId) {
      throw new Error(`Folder membership removal requires VITE_MSM_API_BASE_URL: ${folderId}`);
    },
    async listTags() {
      return [...tags];
    },
    async createTag(request) {
      throw new Error(`Tag creation requires VITE_MSM_API_BASE_URL: ${request.id}`);
    },
    async listPackTags() {
      return [];
    },
    async addTagToPack(packId) {
      throw new Error(`Pack tag membership requires VITE_MSM_API_BASE_URL: ${packId}`);
    },
    async removeTagFromPack(packId) {
      throw new Error(`Pack tag membership removal requires VITE_MSM_API_BASE_URL: ${packId}`);
    },
    async listSubscriptionGroups() {
      return [...groups];
    },
    async createSubscriptionGroup(request) {
      throw new Error(`Subscription group creation requires VITE_MSM_API_BASE_URL: ${request.id}`);
    },
    async listSubscriptionGroupPacks() {
      return [];
    },
    async addPackToSubscriptionGroup(request) {
      throw new Error(`Subscription group membership requires VITE_MSM_API_BASE_URL: ${request.subscriptionGroupId}`);
    },
    async removePackFromSubscriptionGroup(subscriptionGroupId) {
      throw new Error(`Subscription group membership removal requires VITE_MSM_API_BASE_URL: ${subscriptionGroupId}`);
    },
    async listSubscriptionLinks() {
      return [...subscriptionLinks];
    },
    async createSubscriptionLink(request) {
      throw new Error(`Subscription link creation requires VITE_MSM_API_BASE_URL: ${request.id}`);
    },
    async rotateSubscriptionLink(tokenId) {
      throw new Error(`Subscription link rotation requires VITE_MSM_API_BASE_URL: ${tokenId}`);
    },
    async revokeSubscriptionLink(tokenId) {
      throw new Error(`Subscription link revocation requires VITE_MSM_API_BASE_URL: ${tokenId}`);
    },
  };
}

function mockTenantAdminClient(): TenantAdminClient {
  let members: TenantMemberResponse[] = [
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
  ];
  let settings: TenantSettingsResponse = {
    tenantId: "tenant_1",
    name: "Default tenant",
    publicAssetUrl: null,
    localRegistrationEnabled: true,
    createdAt: "2026-05-09T00:00:00Z",
  };
  let users: TenantUserResponse[] = [
    {
      id: "user_1",
      email: "admin@example.test",
      displayName: "Admin",
      isDisabled: false,
      createdAt: "2026-05-09T00:00:00Z",
    },
    {
      id: "user_2",
      email: "member@example.test",
      displayName: "Member",
      isDisabled: false,
      createdAt: "2026-05-09T00:00:00Z",
    },
  ];
  let roles: TenantRoleResponse[] = [
    {
      id: "role_viewer",
      tenantId: "tenant_1",
      name: "Viewers",
      permissions: ["pack.read"],
      createdAt: "2026-05-09T00:00:00Z",
    },
  ];
  let oidcProviders: OidcProviderResponse[] = [
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
  ];

  return {
    async listTenantMembers() {
      return [...members];
    },
    async setTenantMemberRole(tenantId, userId, role) {
      const existing = members.find((member) => member.tenantId === tenantId && member.userId === userId);
      if (existing) {
        existing.role = role;
        return { ...existing };
      }

      const created = {
        tenantId,
        userId,
        role,
        createdAt: "2026-05-09T00:00:00Z",
      };
      members = [...members, created];
      return created;
    },
    async getTenantSettings() {
      return { ...settings };
    },
    async updateTenantSettings(tenantId, request) {
      settings = {
        tenantId,
        name: request.name,
        publicAssetUrl: request.publicAssetUrl,
        localRegistrationEnabled: request.localRegistrationEnabled,
        createdAt: settings.createdAt,
      };
      return { ...settings };
    },
    async setTenantUserStatus(_tenantId, userId, isDisabled) {
      const existing = users.find((user) => user.id === userId);
      if (existing) {
        existing.isDisabled = isDisabled;
        return { ...existing };
      }

      const created = {
        id: userId,
        email: `${userId}@example.test`,
        displayName: userId,
        isDisabled,
        createdAt: "2026-05-09T00:00:00Z",
      };
      users = [...users, created];
      return created;
    },
    async listTenantRoles() {
      return roles.map((role) => ({ ...role, permissions: [...role.permissions] }));
    },
    async upsertTenantRole(tenantId, roleId, request) {
      const existing = roles.find((role) => role.id === roleId && role.tenantId === tenantId);
      if (existing) {
        existing.name = request.name;
        existing.permissions = [...request.permissions];
        return { ...existing, permissions: [...existing.permissions] };
      }

      const created = {
        id: roleId,
        tenantId,
        name: request.name,
        permissions: [...request.permissions],
        createdAt: "2026-05-09T00:00:00Z",
      };
      roles = [...roles, created];
      return { ...created, permissions: [...created.permissions] };
    },
    async listOidcProviders() {
      return oidcProviders.map((provider) => ({ ...provider, scopes: [...provider.scopes] }));
    },
    async upsertOidcProvider(tenantId, providerId, request) {
      const nextProvider = {
        id: providerId,
        tenantId,
        displayName: request.displayName,
        issuerUrl: request.issuerUrl,
        clientId: request.clientId,
        clientSecret: "[redacted]",
        scopes: [...request.scopes],
        isEnabled: request.isEnabled,
        allowRegistration: request.allowRegistration,
        createdAt: "2026-05-10T00:00:00Z",
        updatedAt: "2026-05-10T00:00:00Z",
      };
      oidcProviders = [
        ...oidcProviders.filter((provider) => provider.id !== providerId || provider.tenantId !== tenantId),
        nextProvider,
      ];
      return { ...nextProvider, scopes: [...nextProvider.scopes] };
    },
    async deleteOidcProvider(tenantId, providerId) {
      oidcProviders = oidcProviders.filter(
        (provider) => provider.id !== providerId || provider.tenantId !== tenantId,
      );
    },
  };
}
