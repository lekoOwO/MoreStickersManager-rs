<script setup lang="ts">
import { MenuIcon, MoonIcon, SunIcon } from "lucide-vue-next";
import { computed, ref, watch } from "vue";

import PackDashboard from "@/app/PackDashboard.vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { createPatClient, type CreatedPersonalAccessTokenResponse, type PersonalAccessTokenResponse } from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import type { ThemePreference } from "@/lib/theme";

const props = defineProps<{
  locale: Locale;
  patToken: string;
  theme: ThemePreference;
}>();

const emit = defineEmits<{
  toggleLocale: [];
  toggleTheme: [];
  updatePatToken: [token: string];
}>();

const mobileNavOpen = ref(false);
const tokenDraft = ref(props.patToken);
const tokenId = ref("webui");
const tokenName = ref("Web UI");
const tokenScopes = ref("pack.read import.run pat.manage");
const tokens = ref<PersonalAccessTokenResponse[]>([]);
const createdToken = ref<CreatedPersonalAccessTokenResponse | null>(null);
const patError = ref("");
const labels = computed(() => allMessages()[props.locale]);
const navigationItems = computed(() => [
  labels.value.overview,
  labels.value.packs,
  labels.value.providers,
  labels.value.settings,
]);
const patClient = computed(() => {
  const baseUrl = import.meta.env.VITE_MSM_API_BASE_URL;
  return baseUrl ? createPatClient({ baseUrl, authToken: props.patToken }) : null;
});
const patUserId = computed(() => import.meta.env.VITE_MSM_USER_ID || "demo");

watch(
  () => props.patToken,
  (nextToken) => {
    tokenDraft.value = nextToken;
  },
);

function saveToken() {
  emit("updatePatToken", tokenDraft.value);
}

function clearToken() {
  tokenDraft.value = "";
  createdToken.value = null;
  emit("updatePatToken", "");
}

async function listTokens() {
  patError.value = "";
  try {
    tokens.value = await requirePatClient().listPersonalAccessTokens(patUserId.value);
  } catch (error) {
    patError.value = error instanceof Error ? error.message : String(error);
  }
}

async function createToken() {
  patError.value = "";
  createdToken.value = null;
  try {
    createdToken.value = await requirePatClient().createPersonalAccessToken({
      id: tokenId.value.trim(),
      userId: patUserId.value,
      name: tokenName.value.trim() || "Web UI",
      scopes: tokenScopes.value.split(/[,\s]+/).filter(Boolean),
      expiresAt: null,
    });
    await listTokens();
  } catch (error) {
    patError.value = error instanceof Error ? error.message : String(error);
  }
}

async function revokeToken(tokenId: string) {
  patError.value = "";
  try {
    await requirePatClient().revokePersonalAccessToken(tokenId);
    await listTokens();
  } catch (error) {
    patError.value = error instanceof Error ? error.message : String(error);
  }
}

function requirePatClient() {
  if (!patClient.value) {
    throw new Error("VITE_MSM_API_BASE_URL is not configured");
  }

  return patClient.value;
}
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
          <Card class="mb-6">
            <CardHeader>
              <CardTitle>{{ labels.personalAccessTokens }}</CardTitle>
              <CardDescription>{{ labels.patTokenHelp }}</CardDescription>
            </CardHeader>
            <CardContent class="flex flex-col gap-4">
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.currentPat }}
                <input
                  v-model="tokenDraft"
                  class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :placeholder="labels.patPlaceholder"
                  type="password"
                />
              </label>
              <div class="flex flex-wrap gap-2">
                <Button type="button" @click="saveToken">{{ labels.savePatToken }}</Button>
                <Button type="button" variant="outline" @click="clearToken">{{ labels.clearPatToken }}</Button>
                <Button type="button" variant="secondary" @click="listTokens">{{ labels.refreshTokens }}</Button>
              </div>
              <div class="grid gap-3 md:grid-cols-3">
                <label class="flex flex-col gap-2 text-sm font-medium">
                  {{ labels.tokenId }}
                  <input v-model="tokenId" class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
                </label>
                <label class="flex flex-col gap-2 text-sm font-medium">
                  {{ labels.tokenName }}
                  <input v-model="tokenName" class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
                </label>
                <label class="flex flex-col gap-2 text-sm font-medium">
                  {{ labels.tokenScopes }}
                  <input v-model="tokenScopes" class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
                </label>
              </div>
              <Button type="button" class="w-fit" @click="createToken">{{ labels.createPatToken }}</Button>
              <p v-if="createdToken" class="rounded-lg border bg-background/70 p-3 text-sm">
                {{ labels.createdTokenOnce }} <code class="font-mono">{{ createdToken.token }}</code>
              </p>
              <p v-if="patError" class="rounded-lg border bg-background/70 p-3 text-sm text-muted-foreground">
                {{ patError }}
              </p>
              <article
                v-for="token in tokens"
                :key="token.id"
                class="flex flex-wrap items-center justify-between gap-3 rounded-lg border bg-background/70 p-3"
              >
                <div class="min-w-0">
                  <p class="font-semibold">{{ token.name }}</p>
                  <p class="text-sm text-muted-foreground">{{ token.id }} · {{ token.scopes.join(", ") }}</p>
                </div>
                <div class="flex items-center gap-2">
                  <Badge variant="secondary">{{ token.createdAt.split("T")[0] }}</Badge>
                  <Button type="button" variant="outline" size="sm" @click="revokeToken(token.id)">
                    {{ labels.revokePatToken }}
                  </Button>
                </div>
              </article>
            </CardContent>
          </Card>
          <PackDashboard :locale="locale" :pat-token="patToken" />
        </main>
      </div>
    </div>
  </div>
</template>
