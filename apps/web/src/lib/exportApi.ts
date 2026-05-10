export interface ExportTargetKind {
  kind: string;
  displayName: string;
  supportsRemotePublication: boolean;
  supportsMediaConversion: boolean;
  requiresCredentials: boolean;
}

export interface ExportTarget {
  id: string;
  tenantId: string;
  kind: string;
  name: string;
  config: Record<string, unknown>;
  isEnabled: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateExportTargetRequest {
  id: string;
  tenantId: string;
  kind: string;
  name: string;
  config: Record<string, unknown>;
  isEnabled: boolean;
}

export interface UpdateExportTargetRequest {
  targetId: string;
  name: string;
  config: Record<string, unknown>;
  isEnabled: boolean;
}

export interface ExportJob {
  id: string;
  tenantId: string;
  ownerUserId: string;
  sourcePackId: string;
  targetId: string;
  status: string;
  request: Record<string, unknown>;
  result: Record<string, unknown> | null;
  errorSummary: string | null;
  attemptCount: number;
  maxAttempts: number;
  nextAttemptAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateExportJobRequest {
  id: string;
  tenantId: string;
  sourcePackId: string;
  targetId: string;
  options: Record<string, unknown>;
}

export interface ExportJobEvent {
  jobId: string;
  sequence: number;
  level: string;
  stage: string;
  message: string;
  metadata: Record<string, unknown>;
  createdAt: string;
}

export interface TelegramPublication {
  id: string;
  packId: string;
  targetId: string;
  jobId: string;
  stickerSetName: string;
  stickerSetUrl: string;
  stickerCount: number;
  stickerType: string;
  createdAt: string;
  updatedAt: string;
}

export function exportJobResultLink(result: Record<string, unknown> | null | undefined) {
  if (!result) {
    return "";
  }

  const telegramUrl = readString(result.telegramUrl);
  if (telegramUrl) {
    return telegramUrl;
  }

  const stickerSetUrl = readString(result.stickerSetUrl);
  if (stickerSetUrl) {
    return stickerSetUrl;
  }

  if (result.kind === "telegramPublished") {
    return readString(result.url);
  }

  return readString(result.url);
}

export interface ExportClient {
  listExportTargetKinds(): Promise<ExportTargetKind[]>;
  listExportTargets(tenantId: string): Promise<ExportTarget[]>;
  createExportTarget(request: CreateExportTargetRequest): Promise<ExportTarget>;
  updateExportTarget(request: UpdateExportTargetRequest): Promise<ExportTarget>;
  deleteExportTarget(targetId: string): Promise<void>;
  createExportJob(request: CreateExportJobRequest): Promise<ExportJob>;
  getExportJob(jobId: string): Promise<ExportJob>;
  requeueExportJob(jobId: string): Promise<ExportJob>;
  listExportJobEvents(jobId: string): Promise<ExportJobEvent[]>;
  listTelegramPublications(packId: string): Promise<TelegramPublication[]>;
  getTelegramPublication(publicationId: string): Promise<TelegramPublication>;
}

function readString(value: unknown) {
  return typeof value === "string" ? value : "";
}

export interface ExportClientOptions {
  baseUrl?: string;
  authToken?: string;
  fetchImpl?: typeof fetch;
}

export function createExportClient(options: ExportClientOptions = {}): ExportClient {
  const baseUrl = options.baseUrl?.trim();
  if (!baseUrl) {
    throw new Error("Export API client requires a base URL");
  }

  const fetchImpl = options.fetchImpl ?? fetch;

  return {
    async listExportTargetKinds() {
      const response = await fetchOptional(
        fetchImpl,
        `${trimBaseUrl(baseUrl)}/api/v1/export-target-kinds`,
        authInit(options.authToken),
      );
      await requireOk(response, "list export target kinds");
      return (await response.json()) as ExportTargetKind[];
    },
    async listExportTargets(tenantId) {
      const response = await fetchOptional(fetchImpl, exportTargetListUrl(baseUrl, tenantId), authInit(options.authToken));
      await requireOk(response, "list export targets");
      return (await response.json()) as ExportTarget[];
    },
    async createExportTarget(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/export-targets`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      await requireOk(response, "create export target");
      return (await response.json()) as ExportTarget;
    },
    async updateExportTarget(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/export-targets/${encodeURIComponent(request.targetId)}`, {
        method: "PATCH",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify({
          name: request.name,
          config: request.config,
          isEnabled: request.isEnabled,
        }),
      });
      await requireOk(response, "update export target");
      return (await response.json()) as ExportTarget;
    },
    async deleteExportTarget(targetId) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/export-targets/${encodeURIComponent(targetId)}`, {
        method: "DELETE",
        ...authInit(options.authToken),
      });
      await requireOk(response, "delete export target");
    },
    async createExportJob(request) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/export-jobs`, {
        method: "POST",
        headers: jsonHeaders(options.authToken),
        body: JSON.stringify(request),
      });
      await requireOk(response, "create export job");
      return (await response.json()) as ExportJob;
    },
    async getExportJob(jobId) {
      const response = await fetchOptional(
        fetchImpl,
        `${trimBaseUrl(baseUrl)}/api/v1/export-jobs/${encodeURIComponent(jobId)}`,
        authInit(options.authToken),
      );
      await requireOk(response, "get export job");
      return (await response.json()) as ExportJob;
    },
    async requeueExportJob(jobId) {
      const response = await fetchImpl(`${trimBaseUrl(baseUrl)}/api/v1/export-jobs/${encodeURIComponent(jobId)}/requeue`, {
        method: "POST",
        ...authInit(options.authToken),
      });
      await requireOk(response, "requeue export job");
      return (await response.json()) as ExportJob;
    },
    async listExportJobEvents(jobId) {
      const response = await fetchOptional(
        fetchImpl,
        `${trimBaseUrl(baseUrl)}/api/v1/export-jobs/${encodeURIComponent(jobId)}/events`,
        authInit(options.authToken),
      );
      await requireOk(response, "list export job events");
      return (await response.json()) as ExportJobEvent[];
    },
    async listTelegramPublications(packId) {
      const response = await fetchOptional(
        fetchImpl,
        telegramPublicationListUrl(baseUrl, packId),
        authInit(options.authToken),
      );
      await requireOk(response, "list Telegram publications");
      return (await response.json()) as TelegramPublication[];
    },
    async getTelegramPublication(publicationId) {
      const response = await fetchOptional(
        fetchImpl,
        `${trimBaseUrl(baseUrl)}/api/v1/telegram-publications/${encodeURIComponent(publicationId)}`,
        authInit(options.authToken),
      );
      await requireOk(response, "get Telegram publication");
      return (await response.json()) as TelegramPublication;
    },
  };
}

export function exportTargetListUrl(baseUrl: string, tenantId: string) {
  const query = new URLSearchParams({ tenantId });
  return `${trimBaseUrl(baseUrl)}/api/v1/export-targets?${query.toString()}`;
}

export function telegramPublicationListUrl(baseUrl: string, packId: string) {
  const query = new URLSearchParams({ packId });
  return `${trimBaseUrl(baseUrl)}/api/v1/telegram-publications?${query.toString()}`;
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

async function requireOk(response: Response, action: string) {
  if (!response.ok) {
    throw new Error(`Failed to ${action}: HTTP ${response.status}`);
  }
}
