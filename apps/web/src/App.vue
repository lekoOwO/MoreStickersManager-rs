<script setup lang="ts">
import { ref } from "vue";

import AppShell from "@/app/AppShell.vue";
import { createI18nController, type Locale } from "@/lib/i18n";
import { createThemeController, type ThemePreference } from "@/lib/theme";

const themeController = createThemeController();
const i18nController = createI18nController();

const theme = ref<ThemePreference>(themeController.preference);
const locale = ref<Locale>(i18nController.locale);

function toggleTheme() {
  themeController.toggleResolvedTheme();
  theme.value = themeController.preference;
}

function toggleLocale() {
  const nextLocale = locale.value === "zh-TW" ? "en" : "zh-TW";
  i18nController.setLocale(nextLocale);
  locale.value = nextLocale;
}
</script>

<template>
  <AppShell :locale="locale" :theme="theme" @toggle-locale="toggleLocale" @toggle-theme="toggleTheme" />
</template>
