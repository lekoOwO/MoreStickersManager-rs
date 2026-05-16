<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import ExportJobTimeline from "@/components/ExportJobTimeline.vue";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  createExportClient,
  type ExportClient,
  type ExportJob,
  type ExportJobEvent,
  type ExportTarget,
  type TelegramPublication,
  exportJobResultLink,
} from "@/lib/exportApi";
import { allMessages, type Locale } from "@/lib/i18n";
import { resolveApiBaseUrl } from "@/lib/runtime-config";
import type { StickerPackSummary } from "@/lib/sticker-packs";

const props = defineProps<{
  locale: Locale;
  tenantId: string;
  packs: StickerPackSummary[];
  patToken?: string;
  exportClient?: ExportClient;
}>();

const targets = ref<ExportTarget[]>([]);
const selectedPackId = ref("");
const selectedTargetId = ref("");
const jobId = ref("job_web_export");
const optionsJson = ref("{}");
const dryRun = ref(true);
const reconciliationMode = ref<"none" | "createOnly" | "appendMissing" | "mirror">("none");
const executeReconciliation = ref(false);
const allowDestructiveReconciliation = ref(false);
const telegramOptionsTouched = ref(false);
const activeJob = ref<ExportJob | null>(null);
const events = ref<ExportJobEvent[]>([]);
const publications = ref<TelegramPublication[]>([]);
const publicationLoading = ref(false);
const publicationError = ref("");
const loadError = ref("");
const actionError = ref("");
const labels = computed(() => allMessages()[props.locale]);
const selectedPack = computed(() => props.packs.find((pack) => pack.id === selectedPackId.value) ?? null);
const selectedTarget = computed(() => targets.value.find((target) => target.id === selectedTargetId.value) ?? null);
const isTelegramTarget = computed(() => selectedTarget.value?.kind === "telegram");
const resultLink = computed(() => {
  return exportJobResultLink(activeJob.value?.result);
});

onMounted(loadTargets);
watch(() => props.patToken, loadTargets);
watch(() => props.exportClient, loadTargets);
watch(() => props.patToken, loadPublications);
watch(() => props.exportClient, loadPublications);
watch(() => selectedPackId.value, loadPublications);
watch(
  () => props.packs,
  () => {
    selectedPackId.value = props.packs[0]?.id ?? "";
  },
  { immediate: true },
);

async function loadTargets() {
  loadError.value = "";
  try {
    targets.value = await exportClient().listExportTargets(props.tenantId);
    selectedTargetId.value = targets.value[0]?.id ?? "";
  } catch (error) {
    targets.value = [];
    loadError.value = error instanceof Error ? error.message : String(error);
  }
}

async function queueExportJob() {
  actionError.value = "";
  try {
    const options = exportOptions();
    activeJob.value = await exportClient().createExportJob({
      id: jobId.value.trim(),
      tenantId: props.tenantId,
      sourcePackId: selectedPackId.value,
      targetId: selectedTargetId.value,
      options,
    });
    events.value = await exportClient().listExportJobEvents(activeJob.value.id);
    await loadPublications();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function refreshExportJob() {
  if (!activeJob.value) {
    return;
  }

  actionError.value = "";
  try {
    activeJob.value = await exportClient().getExportJob(activeJob.value.id);
    events.value = await exportClient().listExportJobEvents(activeJob.value.id);
    await loadPublications();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function requeueExportJob() {
  if (!activeJob.value) {
    return;
  }

  actionError.value = "";
  try {
    activeJob.value = await exportClient().requeueExportJob(activeJob.value.id);
    events.value = await exportClient().listExportJobEvents(activeJob.value.id);
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function loadPublications() {
  publicationError.value = "";
  if (!selectedPackId.value) {
    publications.value = [];
    return;
  }

  publicationLoading.value = true;
  try {
    publications.value = await exportClient().listTelegramPublications(selectedPackId.value);
  } catch (error) {
    publications.value = [];
    publicationError.value = error instanceof Error ? error.message : String(error);
  } finally {
    publicationLoading.value = false;
  }
}

function exportClient() {
  return (
    props.exportClient ??
    createExportClient({
      baseUrl: resolveApiBaseUrl(),
      authToken: props.patToken,
    })
  );
}

function exportOptions() {
  const advancedOptions = JSON.parse(optionsJson.value) as Record<string, unknown>;
  if (!isTelegramTarget.value || !telegramOptionsTouched.value) {
    return advancedOptions;
  }

  return {
    ...advancedOptions,
    dryRun: dryRun.value,
    ...(reconciliationMode.value === "none" ? {} : { reconcileMode: reconciliationMode.value }),
    ...(executeReconciliation.value ? { executeReconciliation: true } : {}),
    ...(allowDestructiveReconciliation.value ? { allowDestructiveReconciliation: true } : {}),
  };
}
</script>

<template>
  <div class="grid gap-4 xl:grid-cols-[0.9fr_1.1fr]">
    <Card>
      <CardHeader>
        <CardTitle>{{ labels.exportPack }}</CardTitle>
        <CardDescription>{{ labels.exportPackHelp }}</CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <p v-if="loadError" class="rounded-lg border bg-background/70 p-3 text-sm text-muted-foreground">
          {{ loadError }}
        </p>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.sourcePack }}
          <select
            v-model="selectedPackId"
            class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.sourcePack"
          >
            <option v-for="pack in packs" :key="pack.id" :value="pack.id">{{ pack.title }}</option>
          </select>
        </label>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.exportTarget }}
          <select
            v-model="selectedTargetId"
            class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.exportTarget"
          >
            <option v-for="target in targets" :key="target.id" :value="target.id">{{ target.name }}</option>
          </select>
        </label>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.exportJobId }}
          <input
            v-model="jobId"
            class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.exportJobId"
          />
        </label>
        <section v-if="isTelegramTarget" class="rounded-xl border bg-background/70 p-3 text-sm" :aria-label="labels.telegramReconciliation">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p class="font-semibold">{{ labels.telegramReconciliation }}</p>
              <p class="mt-1 text-muted-foreground">{{ labels.telegramReconciliationHelp }}</p>
            </div>
            <Badge variant="secondary">{{ selectedTarget?.name ?? "Telegram" }}</Badge>
          </div>
          <div class="mt-4 grid gap-3 md:grid-cols-2">
            <label class="flex flex-col gap-2 text-sm font-medium">
              {{ labels.reconciliationMode }}
              <select
                v-model="reconciliationMode"
                class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                :aria-label="labels.reconciliationMode"
                @change="telegramOptionsTouched = true"
              >
                <option value="none">{{ labels.reconciliationModeNone }}</option>
                <option value="createOnly">{{ labels.reconciliationModeCreateOnly }}</option>
                <option value="appendMissing">{{ labels.reconciliationModeAppendMissing }}</option>
                <option value="mirror">{{ labels.reconciliationModeMirror }}</option>
              </select>
            </label>
            <label class="flex cursor-pointer items-start gap-3 rounded-lg border bg-card/60 p-3">
              <input
                v-model="dryRun"
                class="mt-1 size-4 cursor-pointer accent-[var(--primary)]"
                type="checkbox"
                :aria-label="labels.dryRun"
                @change="telegramOptionsTouched = true"
              />
              <span>
                <span class="block font-semibold">{{ labels.dryRun }}</span>
                <span class="mt-1 block text-xs leading-5 text-muted-foreground">{{ labels.dryRunHelp }}</span>
              </span>
            </label>
            <label class="flex cursor-pointer items-start gap-3 rounded-lg border bg-card/60 p-3">
              <input
                v-model="executeReconciliation"
                class="mt-1 size-4 cursor-pointer accent-[var(--primary)]"
                type="checkbox"
                :aria-label="labels.executeReconciliation"
                @change="telegramOptionsTouched = true"
              />
              <span>
                <span class="block font-semibold">{{ labels.executeReconciliation }}</span>
                <span class="mt-1 block text-xs leading-5 text-muted-foreground">{{ labels.executeReconciliationHelp }}</span>
              </span>
            </label>
            <label class="flex cursor-pointer items-start gap-3 rounded-lg border bg-card/60 p-3">
              <input
                v-model="allowDestructiveReconciliation"
                class="mt-1 size-4 cursor-pointer accent-[var(--primary)]"
                type="checkbox"
                :aria-label="labels.allowDestructiveReconciliation"
                @change="telegramOptionsTouched = true"
              />
              <span>
                <span class="block font-semibold">{{ labels.allowDestructiveReconciliation }}</span>
                <span class="mt-1 block text-xs leading-5 text-muted-foreground">{{ labels.allowDestructiveReconciliationHelp }}</span>
              </span>
            </label>
          </div>
        </section>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.exportOptionsJson }}
          <textarea
            v-model="optionsJson"
            class="min-h-28 rounded-md border bg-background px-3 py-2 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.exportOptionsJson"
          />
        </label>
        <article class="rounded-lg border bg-background/70 p-3 text-sm">
          <p class="font-semibold">{{ labels.conversionSummary }}</p>
          <p class="mt-1 text-muted-foreground">
            {{ selectedPack?.title ?? "-" }} - {{ selectedPack?.stickerCount ?? 0 }} stickers -
            {{ selectedTarget?.kind ?? "-" }}
          </p>
          <p class="mt-2 text-muted-foreground">{{ labels.exportPrivacyNotice }}</p>
        </article>
        <section class="rounded-xl border bg-background/70 p-3 text-sm" :aria-label="labels.telegramPublicationHistory">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <div class="flex flex-wrap items-center gap-2">
                <p class="font-semibold">{{ labels.telegramPublicationHistory }}</p>
                <Badge variant="secondary">{{ publications.length }}</Badge>
              </div>
              <p class="mt-1 text-muted-foreground">{{ labels.telegramPublicationHistoryHelp }}</p>
            </div>
            <Button
              type="button"
              size="sm"
              variant="outline"
              :aria-label="labels.refreshPublicationHistory"
              @click="loadPublications"
            >
              {{ labels.refreshPublicationHistory }}
            </Button>
          </div>
          <p v-if="publicationLoading" class="mt-3 rounded-lg border bg-card/60 p-3 text-muted-foreground">
            {{ labels.loadingPublicationHistory }}
          </p>
          <p v-else-if="publicationError" class="mt-3 rounded-lg border bg-card/60 p-3 text-muted-foreground">
            {{ publicationError }}
          </p>
          <p v-else-if="publications.length === 0" class="mt-3 rounded-lg border bg-card/60 p-3 text-muted-foreground">
            {{ labels.noTelegramPublications }}
          </p>
          <div v-else class="mt-3 flex flex-col gap-2">
            <article
              v-for="publication in publications"
              :key="publication.id"
              class="grid gap-3 rounded-lg border bg-card/60 p-3 md:grid-cols-[1fr_auto]"
            >
              <div class="min-w-0">
                <a
                  class="font-medium text-primary"
                  :href="publication.stickerSetUrl"
                  rel="noreferrer"
                  target="_blank"
                >
                  {{ publication.stickerSetName }}
                </a>
                <p class="mt-1 text-muted-foreground">
                  {{ labels.lastPublished }} {{ publication.updatedAt.split("T")[0] }}
                </p>
              </div>
              <Badge variant="outline">{{ publication.stickerCount }} {{ labels.publicationStickers }}</Badge>
            </article>
          </div>
        </section>
        <div class="flex flex-wrap items-center gap-3">
          <Button type="button" :aria-label="labels.queueExportJob" @click="queueExportJob">
            {{ labels.queueExportJob }}
          </Button>
          <Button type="button" variant="outline" :aria-label="labels.refreshExportJob" @click="refreshExportJob">
            {{ labels.refreshExportJob }}
          </Button>
          <Button
            type="button"
            variant="outline"
            :aria-label="labels.requeueExportJob"
            :disabled="!activeJob || !['failed', 'cancelled'].includes(activeJob.status)"
            @click="requeueExportJob"
          >
            {{ labels.requeueExportJob }}
          </Button>
          <p v-if="actionError" class="text-sm text-muted-foreground">{{ actionError }}</p>
        </div>
        <a
          v-if="resultLink"
          class="rounded-lg border bg-background/70 p-3 text-sm font-medium text-primary"
          :href="resultLink"
          rel="noreferrer"
          target="_blank"
        >
          {{ labels.finalExportLink }}: {{ resultLink }}
        </a>
      </CardContent>
    </Card>
    <ExportJobTimeline :locale="locale" :job="activeJob" :events="events" />
  </div>
</template>
