interface RuntimeLocation {
  origin?: string | null;
}

interface ResolveApiBaseUrlOptions {
  envBaseUrl?: string | null;
  isDev?: boolean;
  location?: RuntimeLocation | null;
}

interface RuntimeTokenStorage {
  getItem(key: string): string | null;
}

interface ResolveInitialPatTokenOptions {
  envPat?: string | null;
  isDev?: boolean;
  storage?: RuntimeTokenStorage | null;
  storageKey?: string;
}

export function resolveApiBaseUrl(options: ResolveApiBaseUrlOptions = {}) {
  const envBaseUrl = normalizeApiBaseUrl(options.envBaseUrl ?? import.meta.env.VITE_MSM_API_BASE_URL);
  if (envBaseUrl) {
    return envBaseUrl;
  }

  const isDev = options.isDev ?? import.meta.env.DEV;
  if (isDev) {
    return "";
  }

  const location = options.location ?? (typeof window === "undefined" ? null : window.location);
  const origin = normalizeApiBaseUrl(location?.origin);
  return origin === "null" ? "" : origin;
}

export function normalizeApiBaseUrl(value: string | null | undefined) {
  return value?.trim().replace(/\/+$/, "") ?? "";
}

export function resolveInitialPatToken(options: ResolveInitialPatTokenOptions = {}) {
  const storageKey = options.storageKey ?? "msm.pat";
  const storedToken = options.storage?.getItem(storageKey)?.trim() ?? "";
  if (storedToken) {
    return storedToken;
  }

  const isDev = options.isDev ?? import.meta.env.DEV;
  if (!isDev) {
    return "";
  }

  return options.envPat?.trim() ?? "";
}
