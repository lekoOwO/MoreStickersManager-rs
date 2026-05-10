<script setup lang="ts">
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  DatabaseIcon,
  KeyRoundIcon,
  LanguagesIcon,
  LayoutDashboardIcon,
  LogInIcon,
  MenuIcon,
  MoonIcon,
  SettingsIcon,
  SunIcon,
  TagsIcon,
  UsersRoundIcon,
  XIcon,
} from "lucide-vue-next";
import { computed, ref, watch, type Component } from "vue";

import PackDashboard from "@/app/PackDashboard.vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createLocalAuthClient,
  createOidcAuthClient,
  createPatClient,
  type CreatedPersonalAccessTokenResponse,
  type OidcLoginStartResponse,
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
const sidebarExpanded = ref(false);
const activeSection = ref<WorkspaceSection>("packs");
const authDialogOpen = ref(false);
const accessDialogOpen = ref(false);
const tokenDraft = ref(props.patToken);
const authUserId = ref(import.meta.env.VITE_MSM_USER_ID || "user_1");
const authDisplayName = ref("Leko");
const authEmail = ref("");
const authPassword = ref("");
const authTokenId = ref("webui");
const oidcTenantId = ref(import.meta.env.VITE_MSM_TENANT_ID || "tenant_1");
const oidcProviderId = ref(import.meta.env.VITE_MSM_OIDC_PROVIDER_ID || "google");
const oidcRedirectUri = ref(
  typeof window === "undefined"
    ? "http://127.0.0.1:5173/auth/oidc/callback"
    : `${window.location.origin}/auth/oidc/callback`,
);
const oidcStartResult = ref<OidcLoginStartResponse | null>(null);
const defaultPatScopes = [
  "pack.create",
  "pack.read",
  "pack.update",
  "pack.delete",
  "pack.manage_access",
  "asset.read",
  "import.run",
  "export.read",
  "export.run",
  "export.target.manage",
  "tenant.manage_members",
  "tenant.manage_settings",
  "tenant.manage_users",
  "tenant.manage_roles",
  "subscription.create",
  "subscription.read",
  "pat.manage",
] as const;
type KnownPatScope = (typeof defaultPatScopes)[number];
const authScopes = ref<string[]>([
  "pack.read",
  "pack.update",
  "pack.delete",
  "import.run",
  "export.read",
  "export.run",
  "subscription.create",
  "subscription.read",
]);
const authResult = ref<CreatedPersonalAccessTokenResponse | null>(null);
const authError = ref("");
const tokenId = ref("webui");
const tokenName = ref("Web UI");
const tokenScopes = ref<string[]>([...defaultPatScopes]);
const tokens = ref<PersonalAccessTokenResponse[]>([]);
const createdToken = ref<CreatedPersonalAccessTokenResponse | null>(null);
const patError = ref("");
const scopePolicyAllowedScopes = ref<string[] | null>(null);
const scopePolicyLoading = ref(false);
const scopePolicyError = ref("");
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
      indicatorClass: "bg-primary",
    };
  }

  if (isConnected.value) {
    return {
      label: labels.value.apiNeedsPat,
      help: labels.value.apiNeedsPatHelp,
      badgeVariant: "accent" as const,
      indicatorClass: "bg-accent-foreground",
    };
  }

  return {
    label: labels.value.mockPreview,
    help: labels.value.mockPreviewHelp,
    badgeVariant: "muted" as const,
    indicatorClass: "bg-muted-foreground",
  };
});

const allScopeOptions = computed(() => [
  { key: "pack.create", label: labels.value.scopePackCreate, help: labels.value.scopePackCreateHelp },
  { key: "pack.read", label: labels.value.scopePackRead, help: labels.value.scopePackReadHelp },
  { key: "pack.update", label: labels.value.scopePackUpdate, help: labels.value.scopePackUpdateHelp },
  { key: "pack.delete", label: labels.value.scopePackDelete, help: labels.value.scopePackDeleteHelp },
  { key: "pack.manage_access", label: labels.value.scopePackManageAccess, help: labels.value.scopePackManageAccessHelp },
  { key: "asset.read", label: labels.value.scopeAssetRead, help: labels.value.scopeAssetReadHelp },
  { key: "import.run", label: labels.value.scopeImportRun, help: labels.value.scopeImportRunHelp },
  { key: "export.read", label: labels.value.scopeExportRead, help: labels.value.scopeExportReadHelp },
  { key: "export.run", label: labels.value.scopeExportRun, help: labels.value.scopeExportRunHelp },
  {
    key: "export.target.manage",
    label: labels.value.scopeExportTargetManage,
    help: labels.value.scopeExportTargetManageHelp,
  },
  {
    key: "tenant.manage_members",
    label: labels.value.scopeTenantManageMembers,
    help: labels.value.scopeTenantManageMembersHelp,
  },
  {
    key: "tenant.manage_settings",
    label: labels.value.scopeTenantManageSettings,
    help: labels.value.scopeTenantManageSettingsHelp,
  },
  {
    key: "tenant.manage_users",
    label: labels.value.scopeTenantManageUsers,
    help: labels.value.scopeTenantManageUsersHelp,
  },
  {
    key: "tenant.manage_roles",
    label: labels.value.scopeTenantManageRoles,
    help: labels.value.scopeTenantManageRolesHelp,
  },
  { key: "subscription.create", label: labels.value.scopeSubscriptionCreate, help: labels.value.scopeSubscriptionCreateHelp },
  { key: "subscription.read", label: labels.value.scopeSubscriptionRead, help: labels.value.scopeSubscriptionReadHelp },
  { key: "pat.manage", label: labels.value.scopePatManage, help: labels.value.scopePatManageHelp },
]);
const scopeOptions = computed(() => {
  const allowedScopes = scopePolicyAllowedScopes.value;
  if (!allowedScopes) {
    return allScopeOptions.value;
  }

  const allowed = new Set(allowedScopes);
  return allScopeOptions.value.filter((scope) => allowed.has(scope.key));
});
const scopePolicyStatus = computed(() => {
  if (scopePolicyLoading.value) {
    return labels.value.scopePolicyLoading;
  }
  if (scopePolicyAllowedScopes.value) {
    return labels.value.scopePolicyLive;
  }
  if (scopePolicyError.value) {
    return `${labels.value.scopePolicyFallback} ${scopePolicyError.value}`;
  }
  return labels.value.scopePolicyFallback;
});

const navigationItems = computed<Array<{ key: WorkspaceSection; label: string; icon: Component }>>(() => [
  { key: "overview", label: labels.value.overview, icon: LayoutDashboardIcon },
  { key: "packs", label: labels.value.packs, icon: DatabaseIcon },
  { key: "metadata", label: labels.value.productMetadata, icon: TagsIcon },
  { key: "admin", label: labels.value.tenantAdmin, icon: UsersRoundIcon },
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
const oidcAuthClient = computed(() => {
  const baseUrl = apiBaseUrl.value;
  return baseUrl ? createOidcAuthClient({ baseUrl }) : null;
});
const patUserId = computed(() => import.meta.env.VITE_MSM_USER_ID || "demo");

watch(
  () => props.patToken,
  (nextToken) => {
    tokenDraft.value = nextToken;
  },
);

watch([accessDialogOpen, authDialogOpen, () => props.patToken], ([isAccessOpen, isAuthOpen]) => {
  if (isAccessOpen || isAuthOpen) {
    void loadPatScopePolicy();
  }
});

function selectSection(section: WorkspaceSection) {
  activeSection.value = section;
  mobileNavOpen.value = false;
}

function currentSectionLabel() {
  return activeSection.value === "overview"
    ? labels.value.overview
    : activeSection.value === "packs"
      ? labels.value.packs
      : activeSection.value === "metadata"
        ? labels.value.productMetadata
        : activeSection.value === "admin"
          ? labels.value.tenantAdmin
          : activeSection.value === "exports"
            ? labels.value.exportPack
            : labels.value.exportTargets;
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
      scopes: authScopes.value,
      expiresAt: null,
    });
    emit("updatePatToken", authResult.value.token);
  } catch (error) {
    authError.value = error instanceof Error ? error.message : String(error);
  }
}

async function startOidcLogin() {
  authError.value = "";
  oidcStartResult.value = null;
  try {
    oidcStartResult.value = await requireOidcAuthClient().startOidcLogin({
      tenantId: oidcTenantId.value.trim(),
      providerId: oidcProviderId.value.trim(),
      redirectUri: oidcRedirectUri.value.trim(),
    });
  } catch (error) {
    authError.value = error instanceof Error ? error.message : String(error);
  }
}

async function loadPatScopePolicy() {
  scopePolicyError.value = "";
  if (!patClient.value || !props.patToken.trim()) {
    scopePolicyAllowedScopes.value = null;
    return;
  }

  scopePolicyLoading.value = true;
  try {
    const policy = await patClient.value.getPatScopePolicy(patUserId.value);
    applyAllowedScopes(policy.allowedScopes);
  } catch (error) {
    scopePolicyAllowedScopes.value = null;
    scopePolicyError.value = error instanceof Error ? error.message : String(error);
  } finally {
    scopePolicyLoading.value = false;
  }
}

function applyAllowedScopes(allowedScopes: string[]) {
  const knownAllowedScopes = allowedScopes.filter((scope): scope is KnownPatScope =>
    defaultPatScopes.includes(scope as KnownPatScope),
  );
  scopePolicyAllowedScopes.value = knownAllowedScopes;
  const allowed = new Set(knownAllowedScopes);
  const tokenWasDefault =
    tokenScopes.value.length === defaultPatScopes.length &&
    defaultPatScopes.every((scope) => tokenScopes.value.includes(scope));

  tokenScopes.value = tokenWasDefault
    ? [...knownAllowedScopes]
    : tokenScopes.value.filter((scope) => allowed.has(scope));
  authScopes.value = authScopes.value.filter((scope) => allowed.has(scope));
}

function requireLocalAuthClient() {
  if (!localAuthClient.value) {
    throw new Error("VITE_MSM_API_BASE_URL is not configured");
  }

  return localAuthClient.value;
}

function requireOidcAuthClient() {
  if (!oidcAuthClient.value) {
    throw new Error("VITE_MSM_API_BASE_URL is not configured");
  }

  return oidcAuthClient.value;
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
      scopes: tokenScopes.value,
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
    <div
      class="grid min-h-svh w-full transition-[grid-template-columns] duration-200 ease-out"
      :class="sidebarExpanded ? 'lg:grid-cols-[16rem_minmax(0,1fr)]' : 'lg:grid-cols-[4.75rem_minmax(0,1fr)]'"
    >
      <aside
        class="hidden border-r bg-card/82 px-3 py-4 backdrop-blur-xl lg:flex lg:flex-col lg:gap-4"
        data-testid="desktop-sidebar"
        :data-expanded="sidebarExpanded ? 'true' : 'false'"
      >
        <div
          class="flex items-center"
          :class="sidebarExpanded ? 'justify-between gap-3' : 'flex-col justify-center gap-2'"
        >
          <button
            class="grid size-11 shrink-0 place-items-center rounded-2xl bg-primary text-sm font-black tracking-tight text-primary-foreground shadow-[0_14px_34px_-24px_color-mix(in_oklch,var(--primary)_70%,transparent)]"
            type="button"
            :aria-label="labels.msmOverview"
            @click="selectSection('overview')"
          >
            MS
          </button>
          <Button
            variant="ghost"
            size="icon"
            class="shrink-0"
            type="button"
            data-testid="sidebar-collapse"
            :aria-expanded="sidebarExpanded"
            :aria-label="sidebarExpanded ? labels.collapseSidebar : labels.expandSidebar"
            @click="sidebarExpanded = !sidebarExpanded"
          >
            <ChevronLeftIcon v-if="sidebarExpanded" data-icon="inline-start" />
            <ChevronRightIcon v-else data-icon="inline-start" />
          </Button>
        </div>

        <div v-if="sidebarExpanded" class="rounded-2xl border bg-background/72 px-3 py-3" data-testid="sidebar-brand">
          <p class="text-[0.82rem] font-semibold leading-5 tracking-tight">{{ labels.appName }}</p>
          <p class="mt-1 text-xs text-muted-foreground">{{ labels.desktopWorkspace }}</p>
        </div>

        <div
          class="rounded-2xl border bg-background/72 p-3"
          :class="sidebarExpanded ? '' : 'grid place-items-center'"
          :aria-label="`${labels.runtime}: ${runtimeMode.label}`"
          :title="runtimeMode.label"
          data-testid="runtime-status"
        >
          <template v-if="sidebarExpanded">
            <Badge :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
          </template>
          <span
            v-else
            class="block size-2.5 rounded-full shadow-[0_0_0_4px_color-mix(in_oklch,currentColor_14%,transparent)]"
            :class="runtimeMode.indicatorClass"
            aria-hidden="true"
          />
          <p v-if="sidebarExpanded" class="mt-2 text-xs leading-5 text-muted-foreground">{{ runtimeMode.help }}</p>
        </div>

        <nav class="flex flex-col gap-1" :aria-label="labels.navigation" data-testid="primary-navigation">
          <button
            v-for="item in navigationItems"
            :key="item.key"
            class="group flex items-center rounded-2xl px-3 py-3 text-left text-sm font-semibold text-muted-foreground hover:bg-accent hover:text-accent-foreground"
            :class="[
              activeSection === item.key ? 'bg-accent text-accent-foreground shadow-sm' : '',
              sidebarExpanded ? 'justify-between gap-3' : 'justify-center',
            ]"
            type="button"
            :aria-label="item.label"
            :aria-current="activeSection === item.key ? 'page' : undefined"
            @click="selectSection(item.key)"
          >
            <span class="flex min-w-0 items-center gap-3">
              <component :is="item.icon" class="size-4 shrink-0" />
              <span v-if="sidebarExpanded" class="truncate">{{ item.label }}</span>
            </span>
            <span
              v-if="sidebarExpanded"
              class="size-1.5 rounded-full bg-current opacity-0 transition-opacity group-hover:opacity-45"
              :class="activeSection === item.key ? 'opacity-70' : ''"
            />
          </button>
        </nav>
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
                  {{ currentSectionLabel() }}
                </h2>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <Badge class="hidden md:inline-flex" data-testid="runtime-badge" :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
              <Button variant="outline" size="sm" type="button" @click="authDialogOpen = true">
                <LogInIcon data-icon="inline-start" />
                <span class="hidden sm:inline">{{ labels.localLogin }}</span>
              </Button>
              <Button variant="outline" size="sm" type="button" @click="accessDialogOpen = true">
                <KeyRoundIcon data-icon="inline-start" />
                <span>{{ labels.patShortName }}</span>
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
          <section class="mb-5 rounded-[1.4rem] border bg-card/84 p-4 shadow-[0_18px_56px_-44px_color-mix(in_oklch,var(--primary)_48%,transparent)] lg:hidden">
            <div class="flex items-center justify-between gap-3">
              <Badge data-testid="runtime-badge" :variant="runtimeMode.badgeVariant">{{ runtimeMode.label }}</Badge>
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
      <section class="max-h-[88dvh] w-full max-w-3xl overflow-y-auto rounded-3xl border bg-card p-5 shadow-2xl msm-scrollbar" role="dialog" aria-modal="true" :aria-label="labels.localLogin">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h2 class="text-xl font-semibold">{{ labels.localLogin }}</h2>
            <p class="mt-1 text-sm text-muted-foreground">{{ labels.localLoginHelp }}</p>
          </div>
          <Button variant="ghost" size="icon" type="button" :aria-label="labels.close" @click="authDialogOpen = false">
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
          <fieldset class="md:col-span-2">
            <legend class="text-sm font-medium">{{ labels.tokenScopes }}</legend>
            <p class="mt-1 text-xs text-muted-foreground">{{ labels.tokenScopesHelp }}</p>
            <p class="mt-2 rounded-lg border bg-background/70 px-3 py-2 text-xs text-muted-foreground">
              {{ scopePolicyStatus }}
            </p>
            <div class="mt-3 grid gap-2 md:grid-cols-2">
              <label
                v-for="scope in scopeOptions"
                :key="`auth-${scope.key}`"
                class="flex cursor-pointer items-start gap-3 rounded-xl border bg-background/72 p-3 text-sm hover:border-primary/45 hover:bg-accent/55"
              >
                <input
                  v-model="authScopes"
                  class="mt-1 size-4 cursor-pointer accent-[var(--primary)]"
                  type="checkbox"
                  :value="scope.key"
                />
                <span class="min-w-0">
                  <span class="block font-semibold">{{ scope.label }}</span>
                  <code class="mt-1 block truncate font-mono text-xs text-muted-foreground">{{ scope.key }}</code>
                  <span class="mt-1 block text-xs leading-5 text-muted-foreground">{{ scope.help }}</span>
                </span>
              </label>
            </div>
          </fieldset>
        </div>
        <div class="mt-5 flex flex-wrap gap-2">
          <Button type="button" variant="outline" @click="registerLocalUser">{{ labels.registerLocalUser }}</Button>
          <Button type="button" @click="loginLocalUser">{{ labels.loginLocalUser }}</Button>
        </div>

        <section class="mt-6 border-t pt-5" :aria-label="labels.oidcLogin">
          <div class="flex flex-col gap-1">
            <h3 class="text-base font-semibold">{{ labels.oidcLogin }}</h3>
            <p class="text-sm leading-6 text-muted-foreground">{{ labels.oidcLoginHelp }}</p>
          </div>
          <div class="mt-4 grid gap-3 md:grid-cols-3">
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.oidcTenantId }}
              <input
                v-model="oidcTenantId"
                class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                :aria-label="labels.oidcTenantId"
              />
            </label>
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.oidcProviderId }}
              <input
                v-model="oidcProviderId"
                class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                :aria-label="labels.oidcProviderId"
              />
            </label>
            <label class="flex flex-col gap-2 text-sm font-medium md:col-span-3">
              {{ labels.oidcRedirectUri }}
              <input
                v-model="oidcRedirectUri"
                class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                :aria-label="labels.oidcRedirectUri"
              />
            </label>
          </div>
          <Button type="button" class="mt-4" :aria-label="labels.startOidcLogin" @click="startOidcLogin">
            {{ labels.startOidcLogin }}
          </Button>
          <div v-if="oidcStartResult" class="mt-4 rounded-xl border bg-background/70 p-3 text-sm">
            <a
              class="font-semibold text-primary underline-offset-4 hover:underline"
              :href="oidcStartResult.authorizationUrl"
              target="_blank"
              rel="noreferrer noopener"
            >
              {{ labels.openOidcAuthorizationUrl }}
            </a>
            <dl class="mt-3 grid gap-2 text-xs text-muted-foreground md:grid-cols-3">
              <div>
                <dt class="font-semibold text-foreground">{{ labels.oidcState }}</dt>
                <dd class="break-all font-mono">{{ oidcStartResult.state }}</dd>
              </div>
              <div>
                <dt class="font-semibold text-foreground">{{ labels.oidcNonce }}</dt>
                <dd class="break-all font-mono">{{ oidcStartResult.nonce }}</dd>
              </div>
              <div>
                <dt class="font-semibold text-foreground">{{ labels.oidcExpiresAt }}</dt>
                <dd class="break-all font-mono">{{ oidcStartResult.expiresAt }}</dd>
              </div>
            </dl>
          </div>
        </section>
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
          <Button variant="ghost" size="icon" type="button" :aria-label="labels.close" @click="accessDialogOpen = false">
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
          </div>
          <fieldset>
            <legend class="text-sm font-medium">{{ labels.tokenScopes }}</legend>
            <p class="mt-1 text-xs text-muted-foreground">{{ labels.tokenScopesHelp }}</p>
            <p class="mt-2 rounded-lg border bg-background/70 px-3 py-2 text-xs text-muted-foreground">
              {{ scopePolicyStatus }}
            </p>
            <div class="mt-3 grid gap-2 md:grid-cols-2">
              <label
                v-for="scope in scopeOptions"
                :key="`pat-${scope.key}`"
                class="flex cursor-pointer items-start gap-3 rounded-xl border bg-background/72 p-3 text-sm hover:border-primary/45 hover:bg-accent/55"
              >
                <input
                  v-model="tokenScopes"
                  class="mt-1 size-4 cursor-pointer accent-[var(--primary)]"
                  type="checkbox"
                  :value="scope.key"
                />
                <span class="min-w-0">
                  <span class="block font-semibold">{{ scope.label }}</span>
                  <code class="mt-1 block truncate font-mono text-xs text-muted-foreground">{{ scope.key }}</code>
                  <span class="mt-1 block text-xs leading-5 text-muted-foreground">{{ scope.help }}</span>
                </span>
              </label>
            </div>
          </fieldset>
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
