<script setup lang="ts">
import { MenuIcon, MoonIcon, SunIcon } from "lucide-vue-next";
import { computed, ref } from "vue";

import PackDashboard from "@/app/PackDashboard.vue";
import { Button } from "@/components/ui/button";
import { allMessages, type Locale } from "@/lib/i18n";
import type { ThemePreference } from "@/lib/theme";

const props = defineProps<{
  locale: Locale;
  theme: ThemePreference;
}>();

const emit = defineEmits<{
  toggleLocale: [];
  toggleTheme: [];
}>();

const mobileNavOpen = ref(false);
const labels = computed(() => allMessages()[props.locale]);
const navigationItems = computed(() => [
  labels.value.overview,
  labels.value.packs,
  labels.value.providers,
  labels.value.settings,
]);
</script>

<template>
  <div class="min-h-svh bg-background text-foreground">
    <div class="mx-auto grid min-h-svh max-w-7xl lg:grid-cols-[17rem_1fr]">
      <aside class="hidden border-r bg-card/70 p-5 backdrop-blur lg:flex lg:flex-col lg:gap-8">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground">MSM</p>
          <h1 class="mt-3 text-2xl font-bold tracking-tight">{{ labels.appName }}</h1>
        </div>
        <nav class="flex flex-col gap-2" :aria-label="labels.navigation">
          <a
            v-for="item in navigationItems"
            :key="item"
            class="rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
            href="#"
          >
            {{ item }}
          </a>
        </nav>
      </aside>

      <div class="flex min-w-0 flex-col">
        <header class="sticky top-0 border-b bg-background/82 px-4 py-3 backdrop-blur md:px-8">
          <div class="flex items-center justify-between gap-3">
            <div class="min-w-0">
              <p class="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground lg:hidden">MSM</p>
              <h2 class="truncate text-xl font-bold md:text-2xl">{{ labels.dashboardTitle }}</h2>
            </div>
            <div class="flex items-center gap-2">
              <Button variant="ghost" size="sm" :aria-label="labels.language" @click="emit('toggleLocale')">
                {{ locale === "zh-TW" ? "EN" : "繁中" }}
              </Button>
              <Button variant="outline" size="icon" :aria-label="labels.theme" @click="emit('toggleTheme')">
                <MoonIcon v-if="theme === 'light'" data-icon="inline-start" />
                <SunIcon v-else data-icon="inline-start" />
              </Button>
              <Button
                variant="outline"
                size="icon"
                class="lg:hidden"
                :aria-expanded="mobileNavOpen"
                :aria-label="labels.navigation"
                @click="mobileNavOpen = !mobileNavOpen"
              >
                <MenuIcon data-icon="inline-start" />
              </Button>
            </div>
          </div>
          <nav
            v-if="mobileNavOpen"
            class="mt-3 grid gap-2 rounded-xl border bg-card p-3 lg:hidden"
            :aria-label="labels.navigation"
          >
            <a
              v-for="item in navigationItems"
              :key="item"
              class="rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              href="#"
            >
              {{ item }}
            </a>
          </nav>
        </header>

        <main class="flex-1 px-4 py-6 md:px-8 md:py-8">
          <section class="mb-8 max-w-3xl">
            <p class="text-sm font-medium text-primary">{{ labels.appName }}</p>
            <h1 class="mt-2 text-3xl font-black tracking-tight md:text-5xl">{{ labels.dashboardTitle }}</h1>
            <p class="mt-3 text-base text-muted-foreground md:text-lg">{{ labels.dashboardSubtitle }}</p>
          </section>
          <PackDashboard :locale="locale" />
        </main>
      </div>
    </div>
  </div>
</template>
