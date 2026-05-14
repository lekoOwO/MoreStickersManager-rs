<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import ExportTargetPanel from "@/components/ExportTargetPanel.vue";
import PackExportWizard from "@/components/PackExportWizard.vue";
import PortabilityPanel from "@/components/PortabilityPanel.vue";
import ProductMetadataPanel from "@/components/ProductMetadataPanel.vue";
import ProviderImportPlanner from "@/components/ProviderImportPlanner.vue";
import TenantAdminPanel from "@/components/TenantAdminPanel.vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createPackClient,
  type PackClient,
  type PortabilityClient,
  type ProductMetadataClient,
  type ProviderImportClient,
  type TenantAdminClient,
  type WritablePackVisibility,
} from "@/lib/api-client";
import type { ExportClient } from "@/lib/exportApi";
import { allMessages, type Locale } from "@/lib/i18n";
import { type PackVisibility, type StickerPackSummary } from "@/lib/sticker-packs";
import type { WorkspaceSection } from "@/lib/workspace";

const props = defineProps<{
  locale: Locale;
  patToken?: string;
  packClient?: PackClient;
  metadataClient?: ProductMetadataClient;
  providerImportClient?: ProviderImportClient;
  tenantAdminClient?: TenantAdminClient;
  portabilityClient?: PortabilityClient;
  exportClient?: ExportClient;
  tenantId?: string;
  ownerUserId?: string;
  activeSection?: WorkspaceSection;
}>();

const packs = ref<StickerPackSummary[]>([]);
const loadError = ref("");
const actionError = ref("");
const importError = ref("");
const importDialogOpen = ref(false);
const importPackId = ref("");
const importVisibility = ref<WritablePackVisibility>("private");
const importJson = ref("");
const internalSection = ref<WorkspaceSection>(props.activeSection ?? "packs");
const drafts = ref<Record<string, { title: string; visibility: WritablePackVisibility }>>({});

const labels = computed(() => allMessages()[props.locale]);
const tenantId = computed(() => props.tenantId ?? import.meta.env.VITE_MSM_TENANT_ID ?? "default");
const ownerUserId = computed(() => props.ownerUserId ?? import.meta.env.VITE_MSM_USER_ID ?? "user_1");
const currentSection = computed(() => props.activeSection ?? internalSection.value);
const currentSectionLabel = computed(() =>
  currentSection.value === "overview"
    ? labels.value.overview
    : currentSection.value === "packs"
      ? labels.value.packs
      : currentSection.value === "metadata"
        ? labels.value.productMetadata
        : currentSection.value === "providers"
          ? labels.value.providers
          : currentSection.value === "admin"
            ? labels.value.tenantAdmin
            : currentSection.value === "exports"
              ? labels.value.exportPack
              : currentSection.value === "migration"
                ? labels.value.migration
                : labels.value.exportTargets,
);
const totalStickers = computed(() => packs.value.reduce((sum, pack) => sum + pack.stickerCount, 0));
const publicPackCount = computed(() => packs.value.filter((pack) => pack.visibility === "public").length);
const privatePackCount = computed(() => packs.value.filter((pack) => pack.visibility === "private").length);
const providerCounts = computed(() => {
  return packs.value.reduce<Record<string, number>>((counts, pack) => {
    counts[pack.provider] = (counts[pack.provider] ?? 0) + 1;
    return counts;
  }, {});
});
const sectionTabs = computed<Array<{ key: WorkspaceSection; label: string }>>(() => [
  { key: "overview", label: labels.value.overview },
  { key: "packs", label: labels.value.packs },
  { key: "providers", label: labels.value.providers },
  { key: "metadata", label: labels.value.productMetadata },
  { key: "admin", label: labels.value.tenantAdmin },
  { key: "migration", label: labels.value.migration },
  { key: "exports", label: labels.value.exportPack },
  { key: "targets", label: labels.value.exportTargets },
]);

onMounted(loadPacks);
watch(() => props.patToken, loadPacks);
watch(() => props.packClient, loadPacks);
watch(
  () => props.activeSection,
  (nextSection) => {
    if (nextSection) {
      internalSection.value = nextSection;
    }
  },
);

async function loadPacks() {
  loadError.value = "";
  try {
    packs.value = await packClient().listStickerPacks();
    drafts.value = Object.fromEntries(
      packs.value.map((pack) => [
        pack.id,
        {
          title: pack.title,
          visibility: writableVisibility(pack.visibility),
        },
      ]),
    );
  } catch (error) {
    packs.value = [];
    drafts.value = {};
    loadError.value = error instanceof Error ? error.message : String(error);
  }
}

async function updatePack(pack: StickerPackSummary) {
  actionError.value = "";
  try {
    const draft = drafts.value[pack.id] ?? {
      title: pack.title,
      visibility: writableVisibility(pack.visibility),
    };
    await packClient().updateStickerPack({
      packId: pack.id,
      title: draft.title.trim() || pack.title,
      visibility: draft.visibility,
    });
    await loadPacks();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function importPack() {
  importError.value = "";
  try {
    const pack = JSON.parse(importJson.value) as unknown;
    await packClient().importStickerPack({
      tenantId: tenantId.value,
      ownerUserId: props.ownerUserId ?? import.meta.env.VITE_MSM_USER_ID ?? "user_1",
      packId: importPackId.value.trim(),
      visibility: importVisibility.value,
      pack,
    });
    importJson.value = "";
    importDialogOpen.value = false;
    await loadPacks();
  } catch (error) {
    importError.value = error instanceof Error ? error.message : String(error);
  }
}

async function deletePack(pack: StickerPackSummary) {
  actionError.value = "";
  try {
    await packClient().deleteStickerPack(pack.id);
    await loadPacks();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

function selectSection(section: WorkspaceSection) {
  internalSection.value = section;
}

function packClient() {
  return (
    props.packClient ??
    createPackClient({
      baseUrl: import.meta.env.VITE_MSM_API_BASE_URL,
      userId: import.meta.env.VITE_MSM_USER_ID,
      authToken: props.patToken,
    })
  );
}

function writableVisibility(visibility: PackVisibility): WritablePackVisibility {
  return visibility === "public" ? "public" : "private";
}

function visibilityLabel(visibility: PackVisibility) {
  return labels.value[visibility];
}

function visibilityVariant(visibility: PackVisibility) {
  if (visibility === "public") {
    return "accent";
  }

  if (visibility === "member") {
    return "secondary";
  }

  return "muted";
}
</script>

<template>
  <div class="flex flex-col gap-7 lg:gap-8">
    <section class="grid gap-3 sm:grid-cols-2 xl:grid-cols-[1.2fr_0.8fr_0.8fr_0.8fr]">
      <div class="rounded-[1.45rem] bg-primary px-5 py-5 text-primary-foreground shadow-[0_24px_72px_-54px_color-mix(in_oklch,var(--primary)_62%,transparent)]">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] opacity-80">{{ labels.totalPacks }}</p>
        <p class="mt-3 text-4xl font-semibold tracking-tight">{{ packs.length }}</p>
      </div>
      <div class="rounded-[1.45rem] border bg-card/76 px-5 py-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">{{ labels.managedStickers }}</p>
        <p class="mt-3 text-4xl font-semibold tracking-tight">{{ totalStickers }}</p>
      </div>
      <div class="rounded-[1.45rem] border bg-card/76 px-5 py-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">{{ labels.publicPacks }}</p>
        <p class="mt-3 text-4xl font-semibold tracking-tight">{{ publicPackCount }}</p>
      </div>
      <div class="rounded-[1.45rem] border bg-card/76 px-5 py-5">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">{{ labels.privatePacks }}</p>
        <p class="mt-3 text-4xl font-semibold tracking-tight">{{ privatePackCount }}</p>
      </div>
    </section>

    <div class="flex flex-wrap items-center justify-between gap-3 border-b pb-3">
      <div v-if="!props.activeSection" class="flex flex-wrap gap-2" role="tablist" :aria-label="labels.navigation">
        <button
          v-for="tab in sectionTabs"
          :key="tab.key"
          class="rounded-full px-4 py-2 text-sm font-semibold text-muted-foreground hover:bg-accent hover:text-accent-foreground"
          :class="currentSection === tab.key ? 'bg-primary text-primary-foreground shadow-sm hover:bg-primary hover:text-primary-foreground' : ''"
          role="tab"
          type="button"
          @click="selectSection(tab.key)"
        >
          {{ tab.label }}
        </button>
      </div>
      <div v-else>
        <h2 class="text-lg font-semibold">{{ currentSectionLabel }}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{{ labels.dashboardSubtitle }}</p>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button type="button" variant="outline" @click="loadPacks">{{ labels.refreshTokens }}</Button>
        <Button type="button" :aria-label="labels.openImportDialog" @click="importDialogOpen = true">
          {{ labels.importStickerPack }}
        </Button>
      </div>
    </div>

    <section v-if="currentSection === 'overview'" class="grid gap-5 xl:grid-cols-[0.7fr_1.3fr]">
      <div class="rounded-3xl border bg-card/90 p-5">
        <h2 class="text-lg font-semibold">{{ labels.providerCoverage }}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{{ labels.providerCoverageHelp }}</p>
        <div class="mt-5 flex flex-col divide-y rounded-2xl border bg-background/70">
          <div
            v-for="(count, provider) in providerCounts"
            :key="provider"
            class="flex items-center justify-between gap-3 px-4 py-3"
          >
            <span class="font-medium">{{ provider }}</span>
            <Badge variant="secondary">{{ count }}</Badge>
          </div>
          <p v-if="Object.keys(providerCounts).length === 0" class="px-4 py-3 text-sm text-muted-foreground">
            {{ labels.noExportEvents }}
          </p>
        </div>
      </div>
      <div class="rounded-3xl border bg-card/90 p-5">
        <h2 class="text-lg font-semibold">{{ labels.recentPacks }}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{{ labels.dashboardSubtitle }}</p>
        <div class="mt-5 grid gap-3 lg:grid-cols-2">
          <article
            v-for="pack in packs.slice(0, 6)"
            :key="pack.id"
            class="rounded-2xl border bg-background/70 p-4 hover:border-primary/45"
          >
            <div class="flex flex-wrap items-center gap-2">
              <h3 class="font-semibold">{{ pack.title }}</h3>
              <Badge variant="outline">{{ pack.provider }}</Badge>
            </div>
            <p class="mt-2 text-sm text-muted-foreground">
              {{ pack.stickerCount }} {{ labels.totalStickers }} · {{ labels.updated }} {{ pack.updatedAt }}
            </p>
          </article>
        </div>
      </div>
    </section>

    <section v-show="currentSection === 'packs'" class="overflow-hidden rounded-[1.6rem] border bg-card/88" data-testid="pack-section">
      <div class="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-4">
        <div>
          <h2 class="text-lg font-semibold">{{ labels.recentPacks }}</h2>
          <p class="mt-1 text-sm text-muted-foreground">{{ labels.dashboardSubtitle }}</p>
        </div>
        <Badge variant="secondary">{{ packs.length }} {{ labels.totalPacks }}</Badge>
      </div>
      <div class="grid gap-3 p-3 xl:hidden">
        <p v-if="loadError" class="rounded-2xl border bg-background/70 px-4 py-3 text-sm text-muted-foreground">{{ loadError }}</p>
        <p v-if="actionError" class="rounded-2xl border bg-background/70 px-4 py-3 text-sm text-muted-foreground">{{ actionError }}</p>
        <article
          v-for="pack in packs"
          :key="pack.id"
          class="rounded-[1.25rem] border bg-background/76 p-4"
        >
          <div class="flex items-start justify-between gap-3">
            <div class="min-w-0">
              <h3 class="truncate text-base font-semibold">{{ pack.title }}</h3>
              <p class="mt-1 truncate text-xs text-muted-foreground">{{ pack.id }}</p>
            </div>
            <Badge :variant="visibilityVariant(pack.visibility)">
              {{ visibilityLabel(pack.visibility) }}
            </Badge>
          </div>
          <div class="mt-4 grid grid-cols-2 gap-2 text-sm">
            <div class="rounded-xl bg-muted/60 p-3">
              <p class="text-xs text-muted-foreground">{{ labels.totalStickers }}</p>
              <p class="mt-1 font-semibold">{{ pack.stickerCount }}</p>
            </div>
            <div class="rounded-xl bg-muted/60 p-3">
              <p class="text-xs text-muted-foreground">{{ labels.provider }}</p>
              <p class="mt-1 font-semibold">{{ pack.provider }}</p>
            </div>
          </div>
          <form class="mt-4 grid gap-2" @submit.prevent>
            <label class="sr-only" :for="`mobile-title-${pack.id}`">{{ labels.packTitle }}</label>
            <input
              :id="`mobile-title-${pack.id}`"
              v-model="drafts[pack.id].title"
              class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.packTitle"
            />
            <label class="sr-only" :for="`mobile-visibility-${pack.id}`">{{ labels.packVisibility }}</label>
            <select
              :id="`mobile-visibility-${pack.id}`"
              v-model="drafts[pack.id].visibility"
              class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.packVisibility"
            >
              <option value="public">{{ labels.public }}</option>
              <option value="private">{{ labels.private }}</option>
            </select>
            <div class="mt-1 grid grid-cols-2 gap-2">
              <Button type="button" size="sm" :aria-label="labels.savePackChanges" @click="updatePack(pack)">
                {{ labels.savePackChanges }}
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                :aria-label="labels.deletePack"
                @click="deletePack(pack)"
              >
                {{ labels.deletePack }}
              </Button>
            </div>
          </form>
        </article>
      </div>
      <div class="hidden xl:block">
        <div class="divide-y">
          <p v-if="loadError" class="px-5 py-4 text-sm text-muted-foreground">{{ loadError }}</p>
          <p v-if="actionError" class="px-5 py-4 text-sm text-muted-foreground">{{ actionError }}</p>
          <article
            v-for="pack in packs"
            :key="pack.id"
            class="grid grid-cols-[minmax(12rem,1.4fr)_7rem_7rem_minmax(14rem,0.9fr)] items-start gap-4 px-5 py-4 hover:bg-accent/55"
          >
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <h3 class="truncate text-base font-semibold">{{ pack.title }}</h3>
                <Badge variant="outline">{{ pack.provider }}</Badge>
                <Badge :variant="visibilityVariant(pack.visibility)">
                  {{ visibilityLabel(pack.visibility) }}
                </Badge>
              </div>
              <p class="mt-2 text-sm text-muted-foreground">
                {{ pack.id }} · {{ labels.updated }} {{ pack.updatedAt }}
              </p>
            </div>
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.totalStickers }}</p>
              <p class="mt-2 text-xl font-semibold">{{ pack.stickerCount }}</p>
            </div>
            <div>
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.status }}</p>
              <Badge class="mt-2" :variant="pack.subscriptionReady ? 'accent' : 'muted'">
                {{ labels.subscriptionReady }}
              </Badge>
            </div>
            <form class="grid gap-2" @submit.prevent>
              <div class="grid grid-cols-[1fr_8rem] gap-2">
                <label class="sr-only" :for="`title-${pack.id}`">{{ labels.packTitle }}</label>
                <input
                  :id="`title-${pack.id}`"
                  v-model="drafts[pack.id].title"
                  class="h-9 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.packTitle"
                />
                <label class="sr-only" :for="`visibility-${pack.id}`">{{ labels.packVisibility }}</label>
                <select
                  :id="`visibility-${pack.id}`"
                  v-model="drafts[pack.id].visibility"
                  class="h-9 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.packVisibility"
                >
                  <option value="public">{{ labels.public }}</option>
                  <option value="private">{{ labels.private }}</option>
                </select>
              </div>
              <div class="flex flex-wrap gap-2">
                <Button type="button" size="sm" :aria-label="labels.savePackChanges" @click="updatePack(pack)">
                  {{ labels.savePackChanges }}
                </Button>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  :aria-label="labels.deletePack"
                  @click="deletePack(pack)"
                >
                  {{ labels.deletePack }}
                </Button>
              </div>
            </form>
          </article>
        </div>
      </div>
    </section>

    <section v-show="currentSection === 'exports'">
      <PackExportWizard
        :export-client="exportClient"
        :locale="locale"
        :packs="packs"
        :pat-token="patToken"
        :tenant-id="tenantId"
      />
    </section>

    <section v-show="currentSection === 'providers'">
      <ProviderImportPlanner
        :provider-import-client="providerImportClient"
        :locale="locale"
        :pat-token="patToken"
        :tenant-id="tenantId"
        :owner-user-id="ownerUserId"
      />
    </section>

    <section v-show="currentSection === 'metadata'">
      <ProductMetadataPanel
        :metadata-client="metadataClient"
        :locale="locale"
        :packs="packs"
        :pat-token="patToken"
        :tenant-id="tenantId"
        :owner-user-id="ownerUserId"
      />
    </section>

    <section v-show="currentSection === 'admin'">
      <TenantAdminPanel
        :locale="locale"
        :pat-token="patToken"
        :tenant-admin-client="tenantAdminClient"
        :tenant-id="tenantId"
      />
    </section>

    <section v-show="currentSection === 'migration'">
      <PortabilityPanel
        :locale="locale"
        :pat-token="patToken"
        :portability-client="portabilityClient"
        :tenant-id="tenantId"
        :owner-user-id="ownerUserId"
      />
    </section>

    <section v-show="currentSection === 'targets'">
      <ExportTargetPanel
        :export-client="exportClient"
        :locale="locale"
        :pat-token="patToken"
        :tenant-id="tenantId"
      />
    </section>

    <div v-show="importDialogOpen" class="fixed inset-0 z-40 grid place-items-center bg-foreground/20 p-4 backdrop-blur-sm">
      <section class="w-full max-w-4xl rounded-3xl border bg-card p-5 shadow-2xl" role="dialog" aria-modal="true" :aria-label="labels.importStickerPack">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div>
            <h2 class="text-xl font-semibold">{{ labels.importStickerPack }}</h2>
            <p class="mt-1 text-sm text-muted-foreground">{{ labels.importStickerPackHelp }}</p>
          </div>
          <Button type="button" variant="outline" @click="importDialogOpen = false">{{ labels.close }}</Button>
        </div>
        <div class="mt-5 grid gap-4">
          <div class="grid gap-3 md:grid-cols-[1fr_12rem]">
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.importPackId }}
              <input
                v-model="importPackId"
                class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                :aria-label="labels.importPackId"
              />
            </label>
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.importVisibility }}
              <select
                v-model="importVisibility"
                class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                :aria-label="labels.importVisibility"
              >
                <option value="public">{{ labels.public }}</option>
                <option value="private">{{ labels.private }}</option>
              </select>
            </label>
          </div>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.importPackJson }}
            <textarea
              v-model="importJson"
              class="min-h-52 rounded-lg border bg-background px-3 py-2 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.importPackJson"
            />
          </label>
          <div class="flex flex-wrap items-center gap-3">
            <Button type="button" :aria-label="labels.importStickerPack" @click="importPack">
              {{ labels.importStickerPack }}
            </Button>
            <p v-if="importError" class="text-sm text-muted-foreground">{{ importError }}</p>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
