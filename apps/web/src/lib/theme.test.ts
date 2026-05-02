import { describe, expect, it } from "vitest";

import { createThemeController, type ThemeStorage } from "./theme";

class MemoryStorage implements ThemeStorage {
  private readonly values = new Map<string, string>();

  getItem(key: string) {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string) {
    this.values.set(key, value);
  }
}

describe("theme controller", () => {
  it("defaults to system preference and applies light mode without system dark", () => {
    const storage = new MemoryStorage();
    const root = document.createElement("html");

    const controller = createThemeController(storage, root, false);

    expect(controller.preference).toBe("system");
    expect(root.classList.contains("dark")).toBe(false);
    expect(root.dataset.theme).toBe("system");
  });

  it("persists explicit dark preference and applies the dark class", () => {
    const storage = new MemoryStorage();
    const root = document.createElement("html");

    const controller = createThemeController(storage, root, false);
    controller.setPreference("dark");

    expect(controller.preference).toBe("dark");
    expect(storage.getItem("msm.theme")).toBe("dark");
    expect(root.classList.contains("dark")).toBe(true);
  });

  it("toggles from resolved dark mode to light mode", () => {
    const storage = new MemoryStorage();
    const root = document.createElement("html");

    const controller = createThemeController(storage, root, true);
    controller.toggleResolvedTheme();

    expect(controller.preference).toBe("light");
    expect(root.classList.contains("dark")).toBe(false);
  });
});
