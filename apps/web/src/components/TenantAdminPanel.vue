<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createTenantAdminClient,
  type OidcProviderResponse,
  type TenantAdminClient,
  type TenantMemberResponse,
  type TenantMemberRole,
  type TenantRoleResponse,
  type TenantSettingsResponse,
  type TenantUserResponse,
} from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import { resolveApiBaseUrl } from "@/lib/runtime-config";

const props = defineProps<{
  locale: Locale;
  tenantId: string;
  patToken?: string;
  tenantAdminClient?: TenantAdminClient;
}>();

const members = ref<TenantMemberResponse[]>([]);
const roles = ref<TenantRoleResponse[]>([]);
const oidcProviders = ref<OidcProviderResponse[]>([]);
const settings = ref<TenantSettingsResponse | null>(null);
const loadingError = ref("");
const actionError = ref("");
const memberUserId = ref("");
const memberRole = ref<TenantMemberRole>("user");
const settingsName = ref("");
const publicAssetUrl = ref("");
const localRegistrationEnabled = ref(true);
const statusUserId = ref("");
const userStatus = ref<TenantUserResponse | null>(null);
const roleId = ref("");
const roleName = ref("");
const rolePermissions = ref<string[]>([]);
const oidcProviderId = ref("");
const oidcDisplayName = ref("");
const oidcIssuerUrl = ref("");
const oidcClientId = ref("");
const oidcClientSecret = ref("");
const oidcScopes = ref("openid email");
const oidcEnabled = ref(true);
const oidcAllowRegistration = ref(false);
const labels = computed(() => allMessages()[props.locale]);
const adminCount = computed(() => members.value.filter((member) => member.role === "admin").length);
const userCount = computed(() => members.value.filter((member) => member.role === "user").length);
const rolePermissionOptions = computed(() => [
  { key: "pack.create", label: labels.value.scopePackCreate },
  { key: "pack.read", label: labels.value.scopePackRead },
  { key: "pack.update", label: labels.value.scopePackUpdate },
  { key: "pack.delete", label: labels.value.scopePackDelete },
  { key: "pack.manage_access", label: labels.value.scopePackManageAccess },
  { key: "asset.read", label: labels.value.scopeAssetRead },
  { key: "import.run", label: labels.value.scopeImportRun },
  { key: "provider.import", label: labels.value.scopeProviderImport },
  { key: "export.read", label: labels.value.scopeExportRead },
  { key: "export.run", label: labels.value.scopeExportRun },
  { key: "export.target.manage", label: labels.value.scopeExportTargetManage },
  { key: "tenant.manage_members", label: labels.value.scopeTenantManageMembers },
  { key: "tenant.manage_settings", label: labels.value.scopeTenantManageSettings },
  { key: "tenant.manage_users", label: labels.value.scopeTenantManageUsers },
  { key: "tenant.manage_roles", label: labels.value.scopeTenantManageRoles },
  { key: "subscription.create", label: labels.value.scopeSubscriptionCreate },
  { key: "subscription.read", label: labels.value.scopeSubscriptionRead },
  { key: "pat.manage", label: labels.value.scopePatManage },
]);

onMounted(loadTenantAdminData);
watch(() => props.tenantAdminClient, loadTenantAdminData);
watch(() => props.patToken, loadTenantAdminData);
watch(() => props.tenantId, loadTenantAdminData);

async function loadTenantAdminData() {
  loadingError.value = "";
  try {
    const [nextMembers, nextSettings, nextRoles, nextOidcProviders] = await Promise.all([
      tenantAdminClient().listTenantMembers(props.tenantId),
      tenantAdminClient().getTenantSettings(props.tenantId),
      tenantAdminClient().listTenantRoles(props.tenantId),
      tenantAdminClient().listOidcProviders(props.tenantId),
    ]);
    members.value = nextMembers;
    roles.value = nextRoles;
    oidcProviders.value = nextOidcProviders;
    settings.value = nextSettings;
    settingsName.value = nextSettings.name;
    publicAssetUrl.value = nextSettings.publicAssetUrl ?? "";
    localRegistrationEnabled.value = nextSettings.localRegistrationEnabled;
  } catch (error) {
    members.value = [];
    roles.value = [];
    oidcProviders.value = [];
    loadingError.value = error instanceof Error ? error.message : String(error);
  }
}

async function loadMembers() {
  loadingError.value = "";
  try {
    members.value = await tenantAdminClient().listTenantMembers(props.tenantId);
  } catch (error) {
    members.value = [];
    loadingError.value = error instanceof Error ? error.message : String(error);
  }
}

async function loadRoles() {
  roles.value = await tenantAdminClient().listTenantRoles(props.tenantId);
}

async function loadOidcProviders() {
  oidcProviders.value = await tenantAdminClient().listOidcProviders(props.tenantId);
}

async function setMemberRole(userId = memberUserId.value, role = memberRole.value) {
  actionError.value = "";
  try {
    await tenantAdminClient().setTenantMemberRole(props.tenantId, userId.trim(), role);
    memberUserId.value = "";
    await loadMembers();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function saveTenantSettings() {
  actionError.value = "";
  try {
    settings.value = await tenantAdminClient().updateTenantSettings(props.tenantId, {
      name: settingsName.value.trim(),
      publicAssetUrl: publicAssetUrl.value.trim() || null,
      localRegistrationEnabled: localRegistrationEnabled.value,
    });
    settingsName.value = settings.value.name;
    publicAssetUrl.value = settings.value.publicAssetUrl ?? "";
    localRegistrationEnabled.value = settings.value.localRegistrationEnabled;
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function setUserStatus(isDisabled: boolean) {
  actionError.value = "";
  try {
    userStatus.value = await tenantAdminClient().setTenantUserStatus(
      props.tenantId,
      statusUserId.value.trim(),
      isDisabled,
    );
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function saveRoleTemplate() {
  actionError.value = "";
  try {
    await tenantAdminClient().upsertTenantRole(props.tenantId, roleId.value.trim(), {
      name: roleName.value.trim(),
      permissions: [...rolePermissions.value].sort(),
    });
    roleId.value = "";
    roleName.value = "";
    rolePermissions.value = [];
    await loadRoles();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function saveOidcProvider() {
  actionError.value = "";
  try {
    await tenantAdminClient().upsertOidcProvider(props.tenantId, oidcProviderId.value.trim(), {
      displayName: oidcDisplayName.value.trim(),
      issuerUrl: oidcIssuerUrl.value.trim(),
      clientId: oidcClientId.value.trim(),
      clientSecret: oidcClientSecret.value,
      scopes: oidcScopes.value
        .split(/[,\s]+/)
        .map((scope) => scope.trim())
        .filter(Boolean),
      isEnabled: oidcEnabled.value,
      allowRegistration: oidcAllowRegistration.value,
    });
    oidcProviderId.value = "";
    oidcDisplayName.value = "";
    oidcIssuerUrl.value = "";
    oidcClientId.value = "";
    oidcClientSecret.value = "";
    oidcScopes.value = "openid email";
    oidcEnabled.value = true;
    oidcAllowRegistration.value = false;
    await loadOidcProviders();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function deleteOidcProvider(providerId: string) {
  actionError.value = "";
  try {
    await tenantAdminClient().deleteOidcProvider(props.tenantId, providerId);
    await loadOidcProviders();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

function tenantAdminClient() {
  return (
    props.tenantAdminClient ??
    createTenantAdminClient({
      baseUrl: resolveApiBaseUrl(),
      authToken: props.patToken,
    })
  );
}

function roleVariant(role: TenantMemberRole) {
  return role === "admin" ? "accent" : "secondary";
}

function updateMemberRoleFromEvent(userId: string, event: Event) {
  const target = event.target as HTMLSelectElement;
  void setMemberRole(userId, target.value as TenantMemberRole);
}
</script>

<template>
  <section class="flex flex-col gap-5" data-testid="tenant-admin-section">
    <div class="overflow-hidden rounded-[1.6rem] border bg-card/88">
      <div class="grid gap-4 border-b px-5 py-5 xl:grid-cols-[1fr_auto]">
        <div>
          <p class="text-xs font-semibold uppercase tracking-[0.22em] text-muted-foreground">{{ labels.tenantAdminEyebrow }}</p>
          <h2 class="mt-2 text-xl font-semibold tracking-tight">{{ labels.tenantAdmin }}</h2>
          <p class="mt-1 max-w-3xl text-sm leading-6 text-muted-foreground">{{ labels.tenantAdminHelp }}</p>
        </div>
        <div class="flex flex-wrap items-start gap-2 xl:justify-end">
          <Badge variant="accent">{{ adminCount }} {{ labels.roleAdmin }}</Badge>
          <Badge variant="secondary">{{ userCount }} {{ labels.roleUser }}</Badge>
          <Button type="button" variant="outline" @click="loadTenantAdminData">{{ labels.refreshTokens }}</Button>
        </div>
      </div>

      <p v-if="loadingError || actionError" class="mx-5 mt-4 rounded-2xl border bg-background/70 px-4 py-3 text-sm text-muted-foreground">
        {{ loadingError || actionError }}
      </p>

      <div class="grid gap-0 border-b lg:grid-cols-[minmax(18rem,0.82fr)_minmax(0,1.18fr)]">
        <div class="border-b p-5 lg:border-b-0 lg:border-r">
          <div class="flex flex-col gap-5">
            <form class="flex flex-col gap-3" @submit.prevent>
              <div>
                <h3 class="font-semibold">{{ labels.tenantSettings }}</h3>
                <p class="mt-1 text-sm leading-6 text-muted-foreground">{{ labels.scopeTenantManageSettingsHelp }}</p>
              </div>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.tenantName }}
                <input
                  v-model="settingsName"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.tenantName"
                />
              </label>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.publicAssetUrl }}
                <input
                  v-model="publicAssetUrl"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  placeholder="https://cdn.example.test/msm"
                  :aria-label="labels.publicAssetUrl"
                />
              </label>
              <label class="flex items-center gap-3 rounded-xl border bg-background/70 p-3 text-sm font-medium">
                <input
                  v-model="localRegistrationEnabled"
                  type="checkbox"
                  class="size-4 cursor-pointer accent-primary"
                  :aria-label="labels.localRegistrationEnabled"
                />
                <span>{{ labels.localRegistrationEnabled }}</span>
              </label>
              <Button
                type="button"
                class="w-fit"
                :disabled="!settingsName.trim()"
                :aria-label="labels.saveTenantSettings"
                @click="saveTenantSettings"
              >
                {{ labels.saveTenantSettings }}
              </Button>
            </form>

            <form class="flex flex-col gap-3 border-t pt-5" @submit.prevent>
              <div class="flex items-start justify-between gap-3">
                <div>
                  <h3 class="font-semibold">{{ labels.oidcProviders }}</h3>
                  <p class="mt-1 text-sm leading-6 text-muted-foreground">{{ labels.oidcProvidersHelp }}</p>
                </div>
                <Badge variant="secondary">{{ oidcProviders.length }}</Badge>
              </div>
              <div class="divide-y rounded-2xl border bg-background/70">
                <article v-for="provider in oidcProviders" :key="provider.id" class="flex items-start justify-between gap-3 px-4 py-3">
                  <div class="min-w-0">
                    <p class="truncate text-sm font-semibold">{{ provider.displayName }}</p>
                    <p class="mt-1 truncate font-mono text-xs text-muted-foreground">{{ provider.issuerUrl }}</p>
                    <div class="mt-2 flex flex-wrap gap-2">
                      <Badge :variant="provider.isEnabled ? 'accent' : 'secondary'">
                        {{ provider.isEnabled ? labels.enabled : labels.disabled }}
                      </Badge>
                      <Badge variant="secondary">
                        {{ provider.allowRegistration ? labels.registrationAllowed : labels.loginOnly }}
                      </Badge>
                    </div>
                  </div>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    :aria-label="`${labels.deleteOidcProvider} ${provider.id}`"
                    @click="deleteOidcProvider(provider.id)"
                  >
                    {{ labels.delete }}
                  </Button>
                </article>
                <p v-if="oidcProviders.length === 0" class="px-4 py-3 text-sm text-muted-foreground">{{ labels.noOidcProviders }}</p>
              </div>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.oidcProviderId }}
                <input
                  v-model="oidcProviderId"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.oidcProviderId"
                />
              </label>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.oidcDisplayName }}
                <input
                  v-model="oidcDisplayName"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.oidcDisplayName"
                />
              </label>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.oidcIssuerUrl }}
                <input
                  v-model="oidcIssuerUrl"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  placeholder="https://accounts.google.com"
                  :aria-label="labels.oidcIssuerUrl"
                />
              </label>
              <div class="grid gap-3 xl:grid-cols-2">
                <label class="flex flex-col gap-2 text-sm font-medium">
                  {{ labels.oidcClientId }}
                  <input
                    v-model="oidcClientId"
                    class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                    :aria-label="labels.oidcClientId"
                  />
                </label>
                <label class="flex flex-col gap-2 text-sm font-medium">
                  {{ labels.oidcClientSecret }}
                  <input
                    v-model="oidcClientSecret"
                    class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                    type="password"
                    :aria-label="labels.oidcClientSecret"
                  />
                </label>
              </div>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.oidcScopes }}
                <input
                  v-model="oidcScopes"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  placeholder="openid email profile"
                  :aria-label="labels.oidcScopes"
                />
              </label>
              <div class="grid gap-3 xl:grid-cols-2">
                <label class="flex items-center gap-3 rounded-xl border bg-background/70 p-3 text-sm font-medium">
                  <input v-model="oidcEnabled" type="checkbox" class="size-4 cursor-pointer accent-primary" :aria-label="labels.oidcEnabled" />
                  <span>{{ labels.oidcEnabled }}</span>
                </label>
                <label class="flex items-center gap-3 rounded-xl border bg-background/70 p-3 text-sm font-medium">
                  <input
                    v-model="oidcAllowRegistration"
                    type="checkbox"
                    class="size-4 cursor-pointer accent-primary"
                    :aria-label="labels.oidcAllowRegistration"
                  />
                  <span>{{ labels.oidcAllowRegistration }}</span>
                </label>
              </div>
              <Button
                type="button"
                class="w-fit"
                :disabled="!oidcProviderId.trim() || !oidcDisplayName.trim() || !oidcIssuerUrl.trim() || !oidcClientId.trim() || !oidcClientSecret"
                :aria-label="labels.saveOidcProvider"
                @click="saveOidcProvider"
              >
                {{ labels.saveOidcProvider }}
              </Button>
            </form>

            <form class="flex flex-col gap-3 border-t pt-5" @submit.prevent>
              <div>
                <h3 class="font-semibold">{{ labels.userStatus }}</h3>
                <p class="mt-1 text-sm leading-6 text-muted-foreground">{{ labels.scopeTenantManageUsersHelp }}</p>
              </div>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.statusUserId }}
                <input
                  v-model="statusUserId"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.statusUserId"
                />
              </label>
              <div class="flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  :disabled="!statusUserId.trim()"
                  :aria-label="labels.enableUser"
                  @click="setUserStatus(false)"
                >
                  {{ labels.enableUser }}
                </Button>
                <Button
                  type="button"
                  variant="destructive"
                  :disabled="!statusUserId.trim()"
                  :aria-label="labels.disableUser"
                  @click="setUserStatus(true)"
                >
                  {{ labels.disableUser }}
                </Button>
              </div>
              <p v-if="userStatus" class="rounded-xl border bg-background/70 px-3 py-2 text-sm text-muted-foreground">
                {{ userStatus.displayName }} · {{ userStatus.isDisabled ? labels.disabledUser : labels.activeUser }}
              </p>
            </form>
          </div>
        </div>

        <div class="p-5">
          <div class="grid gap-5 xl:grid-cols-[minmax(16rem,0.88fr)_minmax(0,1.12fr)]">
            <form class="flex flex-col gap-3" @submit.prevent>
              <div>
                <h3 class="font-semibold">{{ labels.roleTemplates }}</h3>
                <p class="mt-1 text-sm leading-6 text-muted-foreground">{{ labels.scopeTenantManageRolesHelp }}</p>
              </div>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.roleTemplateId }}
                <input
                  v-model="roleId"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.roleTemplateId"
                />
              </label>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.roleTemplateName }}
                <input
                  v-model="roleName"
                  class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.roleTemplateName"
                />
              </label>
              <fieldset>
                <legend class="text-sm font-medium">{{ labels.roleTemplatePermissions }}</legend>
                <div class="mt-3 grid gap-2 sm:grid-cols-2">
                  <label
                    v-for="permission in rolePermissionOptions"
                    :key="permission.key"
                    class="flex cursor-pointer items-start gap-2 rounded-xl border bg-background/70 p-2.5 text-sm hover:border-primary/45 hover:bg-accent/45"
                  >
                    <input
                      v-model="rolePermissions"
                      class="mt-1 size-4 cursor-pointer accent-[var(--primary)]"
                      type="checkbox"
                      :value="permission.key"
                      :aria-label="`Permission: ${permission.label}`"
                    />
                    <span class="min-w-0">
                      <span class="block font-semibold">{{ permission.label }}</span>
                      <code class="mt-1 block truncate font-mono text-xs text-muted-foreground">{{ permission.key }}</code>
                    </span>
                  </label>
                </div>
              </fieldset>
              <Button
                type="button"
                class="w-fit"
                :disabled="!roleId.trim() || !roleName.trim() || rolePermissions.length === 0"
                :aria-label="labels.saveRoleTemplate"
                @click="saveRoleTemplate"
              >
                {{ labels.saveRoleTemplate }}
              </Button>
            </form>

            <div class="min-w-0">
              <div class="flex items-center justify-between gap-3">
                <h4 class="font-semibold">{{ labels.roleTemplates }}</h4>
                <Badge variant="secondary">{{ roles.length }}</Badge>
              </div>
              <div class="mt-3 divide-y rounded-2xl border bg-background/70">
                <article v-for="role in roles" :key="role.id" class="px-4 py-3">
                  <div class="flex flex-wrap items-center justify-between gap-2">
                    <p class="font-semibold">{{ role.name }}</p>
                    <code class="font-mono text-xs text-muted-foreground">{{ role.id }}</code>
                  </div>
                  <div class="mt-2 flex flex-wrap gap-1.5">
                    <Badge v-for="permission in role.permissions" :key="`${role.id}:${permission}`" variant="outline">
                      {{ permission }}
                    </Badge>
                  </div>
                </article>
                <p v-if="roles.length === 0" class="px-4 py-3 text-sm text-muted-foreground">{{ labels.noTenantRoles }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <form class="grid gap-3 px-5 py-5 lg:grid-cols-[minmax(12rem,1fr)_12rem_auto]" @submit.prevent>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.memberUserId }}
          <input
            v-model="memberUserId"
            class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.memberUserId"
          />
        </label>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.memberRole }}
          <select
            v-model="memberRole"
            class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.memberRole"
          >
            <option value="admin">{{ labels.roleAdmin }}</option>
            <option value="user">{{ labels.roleUser }}</option>
          </select>
        </label>
        <Button type="button" class="self-end" :disabled="!memberUserId.trim()" :aria-label="labels.setMemberRole" @click="setMemberRole()">
          {{ labels.setMemberRole }}
        </Button>
      </form>

      <div class="hidden divide-y border-t xl:block">
        <article
          v-for="member in members"
          :key="`${member.tenantId}:${member.userId}`"
          class="grid grid-cols-[minmax(14rem,1fr)_10rem_14rem] items-center gap-4 px-5 py-4 hover:bg-accent/45"
        >
          <div class="min-w-0">
            <p class="truncate font-semibold">{{ member.userId }}</p>
            <p class="mt-1 font-mono text-xs text-muted-foreground">{{ member.tenantId }}</p>
          </div>
          <Badge class="w-fit" :variant="roleVariant(member.role)">{{ member.role === "admin" ? labels.roleAdmin : labels.roleUser }}</Badge>
          <form class="flex items-center gap-2" @submit.prevent>
            <select
              class="h-9 min-w-28 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="`${labels.memberRole}: ${member.userId}`"
              :value="member.role"
              @change="updateMemberRoleFromEvent(member.userId, $event)"
            >
              <option value="admin">{{ labels.roleAdmin }}</option>
              <option value="user">{{ labels.roleUser }}</option>
            </select>
          </form>
        </article>
      </div>

      <div class="grid gap-3 border-t p-3 xl:hidden">
        <article
          v-for="member in members"
          :key="`${member.tenantId}:${member.userId}`"
          class="rounded-[1.25rem] border bg-background/76 p-4"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <p class="truncate font-semibold">{{ member.userId }}</p>
              <p class="mt-1 font-mono text-xs text-muted-foreground">{{ member.tenantId }}</p>
            </div>
            <Badge :variant="roleVariant(member.role)">{{ member.role === "admin" ? labels.roleAdmin : labels.roleUser }}</Badge>
          </div>
          <select
            class="mt-4 h-10 w-full rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="`${labels.memberRole}: ${member.userId}`"
            :value="member.role"
            @change="updateMemberRoleFromEvent(member.userId, $event)"
          >
            <option value="admin">{{ labels.roleAdmin }}</option>
            <option value="user">{{ labels.roleUser }}</option>
          </select>
        </article>
        <p v-if="members.length === 0" class="px-2 py-3 text-sm text-muted-foreground">{{ labels.noTenantMembers }}</p>
      </div>
    </div>
  </section>
</template>
