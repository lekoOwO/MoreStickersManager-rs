import { describe, expect, it } from "vitest";

import { createI18nController, type LocaleStorage } from "./i18n";

class MemoryStorage implements LocaleStorage {
  private readonly values = new Map<string, string>();

  getItem(key: string) {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string) {
    this.values.set(key, value);
  }
}

describe("i18n controller", () => {
  it("defaults to Traditional Chinese messages", () => {
    const storage = new MemoryStorage();

    const controller = createI18nController(storage);

    expect(controller.locale).toBe("zh-TW");
    expect(controller.t("dashboardTitle")).toBe("貼圖包管理");
  });

  it("persists English locale and returns English messages", () => {
    const storage = new MemoryStorage();

    const controller = createI18nController(storage);
    controller.setLocale("en");

    expect(controller.locale).toBe("en");
    expect(storage.getItem("msm.locale")).toBe("en");
    expect(controller.t("dashboardTitle")).toBe("Sticker Pack Management");
    expect(document.documentElement.lang).toBe("en");
  });
});
