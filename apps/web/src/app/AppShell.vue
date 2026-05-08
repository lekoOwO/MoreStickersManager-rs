<script setup lang="ts">
import {
  DatabaseIcon,
  KeyRoundIcon,
  LanguagesIcon,
  LayoutDashboardIcon,
  LogInIcon,
  MenuIcon,
  MoonIcon,
  SettingsIcon,
  SunIcon,
  XIcon,
} from "lucide-vue-next";
import { computed, ref, watch, type Component } from "vue";

import PackDashboard from "@/app/PackDashboard.vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createLocalAuthClient,
  createPatClient,
  type CreatedPersonalAccessTokenResponse,
  type PersonalAccessTokenResponse,
} from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import type { ThemePreference } from "@/lib/theme";
import type { WorkspaceSection } from "@/lib/workspace";

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
const activeSection = ref<WorkspaceSection>("packs");
const authDialogOpen = ref(false);
const accessDialogOpen = ref(false);
const tokenDraft = ref(props.patToken);
const authUserId = ref(import.meta.env.VITE_MSM_USER_ID || "user_1");
const authDisplayName = ref("Leko");
const authEmail = ref("");
const authPassword = ref("");
const authTokenId = ref("webui");
const authScopes = ref("pack.read import.run pat.manage export.read export.run export.target.manage");
const authResult = ref<CreatedPersonalAccessTokenResponse | null>(null);
const authError = ref("");
const tokenId = ref("webui");
const tokenName = ref("Web UI");
const tokenScopes = ref("pack.read import.run pat.manage export.read export.run export.target.manage");
const tokens = ref<PersonalAccessTokenResponse[]>([]);
const createdToken = ref<CreatedPersonalAccessTokenResponse | null>(null);
const patError = ref("");
const labels = computed(() => allMessages()[props.locale]);
const isConnected = computed(() => Boolean(import.meta.env.VITE_MSM_API_BASE_URL));

const navigationItems = computed<Array<{ key: WorkspaceSection; label: string; icon: Component }>>(() => [
  { key: "overview", label: labels.value.overview, icon: LayoutDashboardIcon },
  { key: "packs", label: labels.value.packs, icon: DatabaseIcon },
  { key: "exports", label: labels.value.exportPack, icon: SettingsIcon },
  { key: "targets", label: labels.value.exportTargets, icon: KeyRoundIcon },
]);

const patClient = computed(() => {
  const baseUrl = import.meta.env.VITE_MSM_API_BASE_URL;
  return baseUrl ? createPatClient({ baseUrl, authToken: props.patToken }) : null;
});
const localAuthClient = computed(() => {
  const baseUrl = import.meta.env.VITE_MSM_API_BASE_URL;
  return baseUrl ? createLocalAuthClient({ baseUrl }) : null;
});
const patUserId = computed(() => import.meta.env.VITE_MSM_USER_ID || "demo");

watch(
  () => props.patToken,
  (nextToken) => {
    tokenDraft.value = nextToken;
  },
);

function selectSection(section: WorkspaceSection) {
  activeSection.value = section;
  mobileNavOpen.value = false;
}

function saveToken() {
  emit("updatePatToken", tokenDraft.value);
}

function clearToken() {
  tokenDraft.value = "";
  createdToken.value = null;
  emit("updatePatToken", "");
}

async function registerLocalUser() {
  authError.value = "";
  try {
    await requireLocalAuthClient().registerLocalUser({
      id: authUserId.value.trim(),
      email: authEmail.value.trim(),
      displayName: authDisplayName.value.trim() || authUserId.value.trim(),
      password: authPassword.value,
    });
  } catch (error) {
    authError.value = error instanceof Error ? error.message : String(error);
  }
}

async function loginLocalUser() {
  authError.value = "";
  authResult.value = null;
  try {
    authResult.value = await requireLocalAuthClient().loginLocalUser({
      email: authEmail.value.trim(),
      password: authPassword.value,
      tokenId: authTokenId.value.trim(),
      tokenName: "Web UI",
      scopes: authScopes.value.split(/[,\s]+/).filter(Boolean),
      expiresAt: null,
    });
    emit("updatePatToken", authResult.value.token);
  } catch (error) {
    authError.value = error instanceof Error ? error.message : String(error);
  }
}

function requireLocalAuthClient() {
  if (!localAuthClient.value) {
    throw new Error("VITE_MSM_API_BASE_URL is not configured");
  }

  return localAuthClient.value;
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

async function revokeToken(nextTokenId: string) {
  patError.value = "";
  try {
    await requirePatClient().revokePersonalAccessToken(nextTokenId);
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
    <div class="mx-auto grid min-h-svh w-full max-w-[1760px] lg:grid-cols-[18rem_1fr]">
      <aside class="hidden border-r bg-card/90 px-4 py-5 backdrop-blur lg:flex lg:flex-col lg:gap-5">
        <div class="rounded-2xl bg-primary px-4 py-5 text-primary-foreground shadow-[0_18px_42px_-28px_rgba(22,72,180,0.8)]">
          <p class="text-xs font-semibold uppercase tracking-[0.26em] opacity-80">MSM</p>
          <h1 class="mt-3 text-2xl font-semibold tracking-tight">{{ labels.appName }}</h1>
          <p class="mt-2 max-w-[18rem] text-sm opacity-80">{{ labels.dashboardSubtitle }}</p>
        </div>

        <nav class="flex flex-col gap-1" :aria-label="labels.navigation">
          <button
            v-for="item in navigationItems"
            :key="item.key"
            class="group flex items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-semibold text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            :class="activeSection === item.key ? 'bg-accent text-accent-foreground shadow-sm' : ''"
            type="button"
            @click="selectSection(item.key)"
          >
            <component :is="item.icon" class="size-4" />
            <span>{{ item.label }}</span>
          </button>
        </nav>

        <div class="mt-auto rounded-2xl border bg-background/80 p-3">
          <div class="flex items-center justify-between gap-2">
            <span class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {{ isConnected ? "API" : "Preview" }}
            </span>
            <Badge :variant="isConnected ? 'secondary' : 'muted'">
              {{ isConnected ? "Connected" : "Mock" }}
            </Badge>
          </div>
          <p class="mt-2 text-xs leading-5 text-muted-foreground">
            {{ isConnected ? "Protected actions use the browser-local PAT." : "Set VITE_MSM_API_BASE_URL to use live data." }}
          </p>
        </div>
      </aside>

      <div class="flex min-w-0 flex-col">
        <header class="sticky top-0 z-20 border-b bg-background/88 px-4 py-3 backdrop-blur md:px-8">
          <div class="flex items-center justify-between gap-3">
            <div class="flex min-w-0 items-center gap-3">
              <Button
                variant="outline"
                size="icon"
                class="lg:hidden"
                :aria-expanded="mobileNavOpen"
                :aria-label="labels.navigation"
                @click="mobileNavOpen = !mobileNavOpen"
              >
                <MenuIcon v-if="!mobileNavOpen" data-icon="inline-start" />
                <XIcon v-else data-icon="inline-start" />
              </Button>
              <div class="min-w-0">
                <p class="text-xs font-semibold uppercase tracking-[0.24em] text-muted-foreground lg:hidden">MSM</p>
                <h2 class="truncate text-xl font-semibold tracking-tight md:text-2xl">{{ labels.dashboardTitle }}</h2>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <Button variant="outline" size="sm" type="button" @click="authDialogOpen = true">
                <LogInIcon data-icon="inline-start" />
                {{ labels.localLogin }}
              </Button>
              <Button variant="outline" size="sm" type="button" @click="accessDialogOpen = true">
                <KeyRoundIcon data-icon="inline-start" />
                PAT
              </Button>
              <Button variant="ghost" size="sm" :aria-label="labels.language" @click="emit('toggleLocale')">
                <LanguagesIcon data-icon="inline-start" />
                {{ locale === "zh-TW" ? "EN" : "繁中" }}
              </Button>
              <Button variant="outline" size="icon" :aria-label="labels.theme" @click="emit('toggleTheme')">
                <MoonIcon v-if="theme === 'light'" data-icon="inline-start" />
                <SunIcon v-else data-icon="inline-start" />
              </Button>
            </div>
          </div>

          <nav
            v-if="mobileNavOpen"
            class="mt-3 grid gap-2 rounded-2xl border bg-card p-3 shadow-xl lg:hidden"
            :aria-label="labels.navigation"
          >
            <button
              v-for="item in navigationItems"
              :key="item.key"
              class="flex items-center gap-3 rounded-xl px-3 py-2 text-left text-sm font-semibold text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              :class="activeSection === item.key ? 'bg-accent text-accent-foreground' : ''"
              type="button"
              @click="selectSection(item.key)"
            >
              <component :is="item.icon" class="size-4" />
              <span>{{ item.label }}</span>
            </button>
          </nav>
        </header>

        <main class="flex-1 px-4 py-6 md:px-8 md:py-8">
          <section class="mb-6 grid gap-6 xl:grid-cols-[minmax(0,1fr)_24rem] xl:items-end">
            <div>
              <Badge :variant="isConnected ? 'secondary' : 'muted'">
                {{ isConnected ? "Live API" : "Mock preview" }}
              </Badge>
              <h1 class="mt-4 max-w-5xl text-4xl font-semibold tracking-[-0.04em] md:text-6xl">
                {{ labels.dashboardTitle }}
              </h1>
              <p class="mt-4 max-w-3xl text-base leading-7 text-muted-foreground md:text-lg">
                {{ labels.dashboardSubtitle }}
              </p>
            </div>
            <div class="rounded-2xl border bg-card/88 p-4">
              <p class="text-sm font-semibold">{{ props.patToken ? labels.currentPat : labels.patTokenHelp }}</p>
              <p class="mt-2 text-sm text-muted-foreground">
                {{ props.patToken ? "Token is stored in this browser session." : "Create or paste a PAT before running protected actions." }}
              </p>
            </div>
          </section>

          <PackDashboard :active-section="activeSection" :locale="locale" :pat-token="patToken" />
        </main>
      </div>
    </div>

    <div v-show="authDialogOpen" class="fixed inset-0 z-40 grid place-items-center bg-foreground/20 p-4 backdrop-blur-sm">
      <section class="w-full max-w-3xl rounded-3xl border bg-card p-5 shadow-2xl" role="dialog" aria-modal="true" :aria-label="labels.localLogin">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h2 class="text-xl font-semibold">{{ labels.localLogin }}</h2>
            <p class="mt-1 text-sm text-muted-foreground">{{ labels.localLoginHelp }}</p>
          </div>
          <Button variant="ghost" size="icon" type="button" @click="authDialogOpen = false">
            <XIcon data-icon="inline-start" />
          </Button>
        </div>
        <div class="mt-5 grid gap-3 md:grid-cols-2">
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.userId }}
            <input v-model="authUserId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.displayName }}
            <input v-model="authDisplayName" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.email }}
            <input v-model="authEmail" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" type="email" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.password }}
            <input v-model="authPassword" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" type="password" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.tokenId }}
            <input v-model="authTokenId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.tokenScopes }}
            <input v-model="authScopes" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
          </label>
        </div>
        <div class="mt-5 flex flex-wrap gap-2">
          <Button type="button" variant="outline" @click="registerLocalUser">{{ labels.registerLocalUser }}</Button>
          <Button type="button" @click="loginLocalUser">{{ labels.loginLocalUser }}</Button>
        </div>
        <p v-if="authResult" class="mt-4 rounded-xl border bg-background/70 p-3 text-sm">
          {{ labels.loginTokenStored }} <code class="font-mono">{{ authResult.token }}</code>
        </p>
        <p v-if="authError" class="mt-4 rounded-xl border bg-background/70 p-3 text-sm text-muted-foreground">
          {{ authError }}
        </p>
      </section>
    </div>

    <div v-show="accessDialogOpen" class="fixed inset-0 z-40 grid place-items-center bg-foreground/20 p-4 backdrop-blur-sm">
      <section class="max-h-[88dvh] w-full max-w-4xl overflow-y-auto rounded-3xl border bg-card p-5 shadow-2xl msm-scrollbar" role="dialog" aria-modal="true" :aria-label="labels.personalAccessTokens">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h2 class="text-xl font-semibold">{{ labels.personalAccessTokens }}</h2>
            <p class="mt-1 text-sm text-muted-foreground">{{ labels.patTokenHelp }}</p>
          </div>
          <Button variant="ghost" size="icon" type="button" @click="accessDialogOpen = false">
            <XIcon data-icon="inline-start" />
          </Button>
        </div>

        <div class="mt-5 flex flex-col gap-4">
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.currentPat }}
            <input
              v-model="tokenDraft"
              class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
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
              <input v-model="tokenId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
            </label>
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.tokenName }}
              <input v-model="tokenName" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
            </label>
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.tokenScopes }}
              <input v-model="tokenScopes" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" />
            </label>
          </div>
          <Button type="button" class="w-fit" @click="createToken">{{ labels.createPatToken }}</Button>
          <p v-if="createdToken" class="rounded-xl border bg-background/70 p-3 text-sm">
            {{ labels.createdTokenOnce }} <code class="font-mono">{{ createdToken.token }}</code>
          </p>
          <p v-if="patError" class="rounded-xl border bg-background/70 p-3 text-sm text-muted-foreground">
            {{ patError }}
          </p>
          <article
            v-for="token in tokens"
            :key="token.id"
            class="flex flex-wrap items-center justify-between gap-3 rounded-xl border bg-background/70 p-3"
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
        </div>
      </section>
    </div>
  </div>
</template>
