export type ThemePreference = "light" | "dark" | "system";

const STORAGE_KEY = "msm.theme";

export interface ThemeController {
  readonly preference: ThemePreference;
  setPreference(preference: ThemePreference): void;
  toggleResolvedTheme(): void;
}

export interface ThemeStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function createThemeController(
  storage: ThemeStorage = window.localStorage,
  root: HTMLElement = document.documentElement,
  systemDark = false,
): ThemeController {
  let preference = readPreference(storage);

  const apply = () => {
    const resolved = preference === "system" ? (systemDark ? "dark" : "light") : preference;
    root.classList.toggle("dark", resolved === "dark");
    root.dataset.theme = preference;
  };

  apply();

  return {
    get preference() {
      return preference;
    },
    setPreference(nextPreference: ThemePreference) {
      preference = nextPreference;
      storage.setItem(STORAGE_KEY, nextPreference);
      apply();
    },
    toggleResolvedTheme() {
      const isDark = root.classList.contains("dark");
      this.setPreference(isDark ? "light" : "dark");
    },
  };
}

function readPreference(storage: ThemeStorage): ThemePreference {
  const value = storage.getItem(STORAGE_KEY);
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }

  return "system";
}
