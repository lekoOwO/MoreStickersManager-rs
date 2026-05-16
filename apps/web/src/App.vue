<script setup lang="ts">
import { ref } from "vue";

import AppShell from "@/app/AppShell.vue";
import { createI18nController, type Locale } from "@/lib/i18n";
import { resolveInitialPatToken } from "@/lib/runtime-config";
import { createThemeController, type ThemePreference } from "@/lib/theme";

const PAT_STORAGE_KEY = "msm.pat";
const themeController = createThemeController();
const i18nController = createI18nController();

const theme = ref<ThemePreference>(themeController.preference);
const locale = ref<Locale>(i18nController.locale);
const devPatSeed = import.meta.env.DEV ? import.meta.env.VITE_MSM_PAT : "";
const patToken = ref(
  resolveInitialPatToken({
    envPat: devPatSeed,
    isDev: import.meta.env.DEV,
    storage: window.localStorage,
    storageKey: PAT_STORAGE_KEY,
  }),
);

function toggleTheme() {
  themeController.toggleResolvedTheme();
  theme.value = themeController.preference;
}

function toggleLocale() {
  const nextLocale = locale.value === "zh-TW" ? "en" : "zh-TW";
  i18nController.setLocale(nextLocale);
  locale.value = nextLocale;
}

function updatePatToken(nextToken: string) {
  const trimmed = nextToken.trim();
  patToken.value = trimmed;
  if (trimmed) {
    window.localStorage.setItem(PAT_STORAGE_KEY, trimmed);
  } else {
    window.localStorage.removeItem(PAT_STORAGE_KEY);
  }
}
</script>

<template>
  <AppShell
    :locale="locale"
    :pat-token="patToken"
    :theme="theme"
    @toggle-locale="toggleLocale"
    @toggle-theme="toggleTheme"
    @update-pat-token="updatePatToken"
  />
</template>
