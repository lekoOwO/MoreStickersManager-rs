export type Locale = "zh-TW" | "en";

const STORAGE_KEY = "msm.locale";

const messages = {
  "zh-TW": {
    appName: "MoreStickersManager",
    dashboardTitle: "貼圖包管理",
    dashboardSubtitle: "集中管理可匯出到 moreStickers 的貼圖包、Provider 與訂閱狀態。",
    totalPacks: "貼圖包",
    totalStickers: "貼圖",
    publicPacks: "公開貼圖包",
    privatePacks: "私有貼圖包",
    providerCoverage: "Provider 覆蓋",
    recentPacks: "近期貼圖包",
    managedStickers: "受管理貼圖",
    subscriptionReady: "訂閱準備",
    updated: "更新",
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
    private: "私有",
    member: "成員可用",
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

export function createI18nController(
  storage: LocaleStorage = window.localStorage,
): I18nController {
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
