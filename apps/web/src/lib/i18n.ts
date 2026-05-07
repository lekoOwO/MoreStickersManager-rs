export type Locale = "zh-TW" | "en";

const STORAGE_KEY = "msm.locale";

const messages = {
  "zh-TW": {
    appName: "MoreStickersManager",
    dashboardTitle: "貼圖包管理",
    dashboardSubtitle: "管理 moreStickers 匯出的貼圖包、Provider 與訂閱狀態。",
    totalPacks: "貼圖包",
    totalStickers: "貼圖",
    publicPacks: "公開貼圖包",
    privatePacks: "私人貼圖包",
    providerCoverage: "Provider 覆蓋",
    recentPacks: "近期貼圖包",
    managedStickers: "已管理貼圖",
    subscriptionReady: "訂閱就緒",
    updated: "更新於",
    theme: "主題",
    language: "語言",
    navigation: "導覽",
    overview: "總覽",
    packs: "貼圖包",
    providers: "Providers",
    settings: "設定",
    light: "亮色",
    dark: "暗色",
    public: "公開",
    private: "私人",
    member: "成員",
    personalAccessTokens: "Personal Access Tokens",
    patTokenHelp: "在瀏覽器本機儲存 PAT，用於受保護的 API 操作。",
    currentPat: "目前 PAT",
    patPlaceholder: "msm_pat_...",
    savePatToken: "儲存 token",
    clearPatToken: "清除 token",
    refreshTokens: "重新整理 token",
    createPatToken: "建立 PAT",
    revokePatToken: "撤銷",
    createdTokenOnce: "已建立 token，僅顯示一次：",
    tokenId: "Token ID",
    tokenName: "Token 名稱",
    tokenScopes: "Scopes",
    localLogin: "本地登入",
    localLoginHelp: "註冊或登入本地帳號；登入成功會自動儲存 PAT。",
    registerLocalUser: "註冊本地帳號",
    loginLocalUser: "登入",
    loginTokenStored: "登入 token 已儲存，僅顯示一次：",
    userId: "User ID",
    displayName: "顯示名稱",
    email: "Email",
    password: "密碼",
    packTitle: "Pack title",
    packVisibility: "Pack visibility",
    savePackChanges: "Save pack changes",
    deletePack: "Delete pack",
    importStickerPack: "Import sticker pack",
    importStickerPackHelp: "Paste a MoreStickers .stickerpack JSON export and import it into MSM.",
    importPackId: "Import pack ID",
    importVisibility: "Import visibility",
    importPackJson: "Import pack JSON",
    exportTargets: "匯出目標",
    exportTargetsHelp: "設定 MoreStickers、Telegram 與未來輸出平台使用的匯出目標。",
    exportTargetKinds: "匯出目標類型",
    targetId: "Target ID",
    targetKind: "Target kind",
    targetName: "Target name",
    targetConfigJson: "Target config JSON",
    targetEnabled: "啟用匯出目標",
    createExportTarget: "Create export target",
    exportPack: "匯出貼圖包",
    exportPackHelp: "選擇貼圖包與匯出目標，建立可由背景 worker 處理的匯出工作。",
    sourcePack: "Source pack",
    exportTarget: "Export target",
    exportOptionsJson: "Export options JSON",
    exportJobId: "Export job ID",
    queueExportJob: "Queue export job",
    refreshExportJob: "Refresh export job",
    conversionSummary: "轉換摘要",
    exportPrivacyNotice: "私有貼圖包匯出時，圖片資源仍會依目標與 MSM 權限設定控管。",
    finalExportLink: "Export result link",
    exportJobTimeline: "Export job timeline",
    exportJobStatus: "Export job status",
    noExportTargets: "尚未設定匯出目標。",
    noExportEvents: "尚無匯出事件。",
    enabled: "啟用",
    disabled: "停用",
    supportsConversion: "支援轉換",
    directExport: "直接匯出",
    remotePublication: "遠端發布",
    localExport: "本地輸出",
    invalidTelegramToken: "Telegram bot token 必須符合 123:token 格式。",
  },
  en: {
    appName: "MoreStickersManager",
    dashboardTitle: "Sticker Pack Management",
    dashboardSubtitle: "Manage packs, providers, and subscription readiness for moreStickers exports.",
    totalPacks: "Packs",
    totalStickers: "Stickers",
    publicPacks: "Public Packs",
    privatePacks: "Private Packs",
    providerCoverage: "Provider Coverage",
    recentPacks: "Recent Packs",
    managedStickers: "Managed Stickers",
    subscriptionReady: "Subscription Ready",
    updated: "Updated",
    theme: "Theme",
    language: "Language",
    navigation: "Navigation",
    overview: "Overview",
    packs: "Packs",
    providers: "Providers",
    settings: "Settings",
    light: "Light",
    dark: "Dark",
    public: "Public",
    private: "Private",
    member: "Members",
    personalAccessTokens: "Personal Access Tokens",
    patTokenHelp: "Store a browser-local PAT for protected API operations.",
    currentPat: "Current PAT",
    patPlaceholder: "msm_pat_...",
    savePatToken: "Save token",
    clearPatToken: "Clear token",
    refreshTokens: "Refresh tokens",
    createPatToken: "Create PAT",
    revokePatToken: "Revoke",
    createdTokenOnce: "Created token, shown once:",
    tokenId: "Token ID",
    tokenName: "Token name",
    tokenScopes: "Scopes",
    localLogin: "Local Login",
    localLoginHelp: "Register or log in with a local account. Successful login stores the returned PAT.",
    registerLocalUser: "Register local user",
    loginLocalUser: "Log in",
    loginTokenStored: "Login token stored, shown once:",
    userId: "User ID",
    displayName: "Display name",
    email: "Email",
    password: "Password",
    packTitle: "Pack title",
    packVisibility: "Pack visibility",
    savePackChanges: "Save pack changes",
    deletePack: "Delete pack",
    importStickerPack: "Import sticker pack",
    importStickerPackHelp: "Paste a MoreStickers .stickerpack JSON export and import it into MSM.",
    importPackId: "Import pack ID",
    importVisibility: "Import visibility",
    importPackJson: "Import pack JSON",
    exportTargets: "Export targets",
    exportTargetsHelp: "Configure export targets for MoreStickers, Telegram, and future output platforms.",
    exportTargetKinds: "Export target kinds",
    targetId: "Target ID",
    targetKind: "Target kind",
    targetName: "Target name",
    targetConfigJson: "Target config JSON",
    targetEnabled: "Target enabled",
    createExportTarget: "Create export target",
    exportPack: "Export sticker pack",
    exportPackHelp: "Choose a sticker pack and export target, then queue a background export job.",
    sourcePack: "Source pack",
    exportTarget: "Export target",
    exportOptionsJson: "Export options JSON",
    exportJobId: "Export job ID",
    queueExportJob: "Queue export job",
    refreshExportJob: "Refresh export job",
    conversionSummary: "Conversion summary",
    exportPrivacyNotice: "Private pack exports still depend on target behavior and MSM asset authorization settings.",
    finalExportLink: "Export result link",
    exportJobTimeline: "Export job timeline",
    exportJobStatus: "Export job status",
    noExportTargets: "No export targets configured yet.",
    noExportEvents: "No export events yet.",
    enabled: "Enabled",
    disabled: "Disabled",
    supportsConversion: "Supports conversion",
    directExport: "Direct export",
    remotePublication: "Remote publication",
    localExport: "Local export",
    invalidTelegramToken: "Telegram bot token must match the 123:token format.",
  },
} as const;

export type MessageKey = keyof (typeof messages)["zh-TW"];

export interface I18nController {
  readonly locale: Locale;
  setLocale(locale: Locale): void;
  t(key: MessageKey): string;
}

export interface LocaleStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function createI18nController(storage: LocaleStorage = window.localStorage): I18nController {
  let locale = readLocale(storage);

  return {
    get locale() {
      return locale;
    },
    setLocale(nextLocale: Locale) {
      locale = nextLocale;
      storage.setItem(STORAGE_KEY, nextLocale);
      document.documentElement.lang = nextLocale === "zh-TW" ? "zh-Hant" : "en";
    },
    t(key: MessageKey) {
      return messages[locale][key];
    },
  };
}

export function allMessages() {
  return messages;
}

function readLocale(storage: LocaleStorage): Locale {
  return storage.getItem(STORAGE_KEY) === "en" ? "en" : "zh-TW";
}
