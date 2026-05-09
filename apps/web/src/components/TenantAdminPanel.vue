<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createTenantAdminClient,
  type TenantAdminClient,
  type TenantMemberResponse,
  type TenantMemberRole,
} from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";

const props = defineProps<{
  locale: Locale;
  tenantId: string;
  patToken?: string;
  tenantAdminClient?: TenantAdminClient;
}>();

const members = ref<TenantMemberResponse[]>([]);
const loadingError = ref("");
const actionError = ref("");
const memberUserId = ref("");
const memberRole = ref<TenantMemberRole>("user");
const labels = computed(() => allMessages()[props.locale]);
const adminCount = computed(() => members.value.filter((member) => member.role === "admin").length);
const userCount = computed(() => members.value.filter((member) => member.role === "user").length);

onMounted(loadMembers);
watch(() => props.tenantAdminClient, loadMembers);
watch(() => props.patToken, loadMembers);
watch(() => props.tenantId, loadMembers);

async function loadMembers() {
  loadingError.value = "";
  try {
    members.value = await tenantAdminClient().listTenantMembers(props.tenantId);
  } catch (error) {
    members.value = [];
    loadingError.value = error instanceof Error ? error.message : String(error);
  }
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

function tenantAdminClient() {
  return (
    props.tenantAdminClient ??
    createTenantAdminClient({
      baseUrl: import.meta.env.VITE_MSM_API_BASE_URL,
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
          <Button type="button" variant="outline" @click="loadMembers">{{ labels.refreshTokens }}</Button>
        </div>
      </div>

      <p v-if="loadingError || actionError" class="mx-5 mt-4 rounded-2xl border bg-background/70 px-4 py-3 text-sm text-muted-foreground">
        {{ loadingError || actionError }}
      </p>

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
