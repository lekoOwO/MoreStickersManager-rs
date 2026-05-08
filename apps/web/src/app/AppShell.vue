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
const apiBaseUrl = computed(() => import.meta.env.VITE_MSM_API_BASE_URL?.trim() ?? "");
const isConnected = computed(() => Boolean(apiBaseUrl.value));
const hasPat = computed(() => Boolean(props.patToken.trim()));
const runtimeMode = computed(() => {
  if (isConnected.value && hasPat.value) {
    return {
      label: labels.value.apiLive,
      help: labels.value.liveApiHelp,
      badgeVariant: "secondary" as const,
      tone: "bg-primary text-primary-foreground",
    };
  }

  if (isConnected.value) {
    return {
      label: labels.value.apiNeedsPat,
      help: labels.value.apiNeedsPatHelp,
      badgeVariant: "accent" as const,
      tone: "bg-accent text-accent-foreground",
    };
  }

  return {
    label: labels.value.mockPreview,
    help: labels.value.mockPreviewHelp,
    badgeVariant: "muted" as const,
    tone: "bg-muted text-muted-foreground",
  };
});

const navigationItems = computed<Array<{ key: WorkspaceSection; label: string; icon: Component }>>(() => [
  { key: "overview", label: labels.value.overview, icon: LayoutDashboardIcon },
  { key: "packs", label: labels.value.packs, icon: DatabaseIcon },
  { key: "exports", label: labels.value.exportPack, icon: SettingsIcon },
  { key: "targets", label: labels.value.exportTargets, icon: KeyRoundIcon },
]);

const patClient = computed(() => {
  const baseUrl = apiBaseUrl.value;
  return baseUrl ? createPatClient({ baseUrl, authToken: props.patToken }) : null;
});
const localAuthClient = computed(() => {
  const baseUrl = apiBaseUrl.value;
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
    <div class="grid min-h-svh w-full lg:grid-cols-[5.25rem_22rem_minmax(0,1fr)]">
      <aside class="hidden border-r bg-card/80 px-3 py-5 backdrop-blur-xl lg:flex lg:flex-col lg:items-center lg:gap-4">
        <button
          class="grid size-12 place-items-center rounded-2xl bg-primary text-base font-black tracking-tight text-primary-foreground shadow-[0_18px_44px_-26px_color-mix(in_oklch,var(--primary)_70%,transparent)]"
          type="button"
          @click="selectSection('overview')"
        >
          MS
        </button>

        <nav class="mt-4 flex flex-col gap-2" :aria-label="labels.navigation">
          <button
            v-for="item in navigationItems"
            :key="item.key"
            class="grid size-11 place-items-center rounded-2xl text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            :class="activeSection === item.key ? 'bg-primary text-primary-foreground shadow-sm hover:bg-primary hover:text-primary-foreground' : ''"
            type="button"
            @click="selectSection(item.key)"
          >
            <component :is="item.icon" class="size-5" />
            <span class="sr-only">{{ item.label }}</span>
          </button>
        </nav>

        <div class="mt-auto flex flex-col gap-2">
          <Button variant="ghost" size="icon" :aria-label="labels.language" @click="emit('toggleLocale')">
            <LanguagesIcon data-icon="inline-start" />
          </Button>
          <Button variant="outline" size="icon" :aria-label="labels.theme" @click="emit('toggleTheme')">
            <MoonIcon v-if="theme === 'light'" data-icon="inline-start" />
            <SunIcon v-else data-icon="inline-start" />
          </Button>
        </div>
      </aside>

      <aside class="hidden border-r bg-card/72 px-5 py-6 backdrop-blur-xl lg:flex lg:flex-col lg:gap-6">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.28em] text-muted-foreground">MSM</p>
          <h1 class="mt-3 break-words text-[1.7rem] font-semibold leading-none tracking-[-0.05em]">
            {{ labels.appName }}
          </h1>
          <p class="mt-4 text-sm leading-6 text-muted-foreground">{{ labels.dashboardSubtitle }}</p>
        </div>

        <div class="rounded-[1.35rem] border bg-background/78 p-4 shadow-[inset_0_1px_0_color-mix(in_oklch,var(--foreground)_8%,transparent)]">
          <div class="flex items-center justify-between gap-3">
            <span class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">Runtime</span>
            <Badge :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
          </div>
          <p class="mt-3 text-xs leading-5 text-muted-foreground">{{ runtimeMode.help }}</p>
          <p v-if="apiBaseUrl" class="mt-3 truncate font-mono text-[0.7rem] text-muted-foreground">{{ apiBaseUrl }}</p>
        </div>

        <nav class="flex flex-col gap-1" :aria-label="labels.navigation">
          <button
            v-for="item in navigationItems"
            :key="item.key"
            class="group flex items-center justify-between gap-3 rounded-2xl px-3 py-3 text-left text-sm font-semibold text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            :class="activeSection === item.key ? 'bg-accent text-accent-foreground shadow-sm' : ''"
            type="button"
            @click="selectSection(item.key)"
          >
            <span class="flex min-w-0 items-center gap-3">
              <component :is="item.icon" class="size-4 shrink-0" />
              <span class="truncate">{{ item.label }}</span>
            </span>
            <span class="size-1.5 rounded-full bg-current opacity-0 transition-opacity group-hover:opacity-45" :class="activeSection === item.key ? 'opacity-70' : ''" />
          </button>
        </nav>

        <div class="mt-auto grid gap-3">
          <Button variant="outline" type="button" class="w-full justify-start" @click="authDialogOpen = true">
            <LogInIcon data-icon="inline-start" />
            {{ labels.localLogin }}
          </Button>
          <Button variant="secondary" type="button" class="w-full justify-start" @click="accessDialogOpen = true">
            <KeyRoundIcon data-icon="inline-start" />
            {{ labels.personalAccessTokens }}
          </Button>
        </div>
      </aside>

      <div class="flex min-w-0 flex-col">
        <header class="sticky top-0 z-20 border-b bg-background/88 px-4 py-3 backdrop-blur md:px-8 lg:px-10">
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
                <h2 class="truncate text-xl font-semibold tracking-tight md:text-2xl lg:text-lg">
                  {{ activeSection === "overview" ? labels.overview : activeSection === "packs" ? labels.packs : activeSection === "exports" ? labels.exportPack : labels.exportTargets }}
                </h2>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <Badge class="hidden md:inline-flex" :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
              <Button variant="outline" size="sm" type="button" @click="authDialogOpen = true">
                <LogInIcon data-icon="inline-start" />
                <span class="hidden sm:inline">{{ labels.localLogin }}</span>
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

        <main class="flex-1 px-4 py-5 md:px-8 md:py-7 lg:px-10 xl:px-12">
          <section class="mb-7 hidden grid-cols-[minmax(0,1.35fr)_minmax(24rem,0.65fr)] gap-6 lg:grid">
            <div>
              <div class="flex flex-wrap items-center gap-3">
                <Badge :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
                <span class="text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground">
                  {{ labels.desktopWorkspace }}
                </span>
              </div>
              <h1 class="mt-4 max-w-[12ch] text-5xl font-semibold leading-[0.92] tracking-[-0.06em] xl:text-7xl">
                {{ labels.dashboardTitle }}
              </h1>
              <p class="mt-5 max-w-2xl text-base leading-7 text-muted-foreground">
                {{ labels.dashboardSubtitle }}
              </p>
            </div>
            <div class="grid content-end gap-4">
              <div class="rounded-[1.6rem] border bg-card/86 p-5 shadow-[0_28px_80px_-56px_color-mix(in_oklch,var(--primary)_55%,transparent)]">
                <p class="text-sm font-semibold">{{ hasPat ? labels.currentPat : labels.patTokenHelp }}</p>
                <p class="mt-2 text-sm leading-6 text-muted-foreground">
                  {{ runtimeMode.help }}
                </p>
                <div class="mt-5 flex flex-wrap gap-2">
                  <Button variant="secondary" size="sm" type="button" @click="accessDialogOpen = true">PAT</Button>
                  <Button variant="outline" size="sm" type="button" @click="authDialogOpen = true">{{ labels.localLogin }}</Button>
                </div>
              </div>
              <div class="h-2 rounded-full bg-muted">
                <div class="h-full rounded-full transition-all" :class="runtimeMode.tone" :style="{ width: hasPat && isConnected ? '100%' : isConnected ? '62%' : '32%' }" />
              </div>
            </div>
          </section>

          <section class="mb-5 rounded-[1.4rem] border bg-card/84 p-4 shadow-[0_18px_56px_-44px_color-mix(in_oklch,var(--primary)_48%,transparent)] lg:hidden">
            <div class="flex items-center justify-between gap-3">
              <Badge :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
              <span class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.mobileWorkspace }}</span>
            </div>
            <h1 class="mt-4 text-3xl font-semibold leading-tight tracking-[-0.045em]">{{ labels.dashboardTitle }}</h1>
            <p class="mt-3 text-sm leading-6 text-muted-foreground">{{ runtimeMode.help }}</p>
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
