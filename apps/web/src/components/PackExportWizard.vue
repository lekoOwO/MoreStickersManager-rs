<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import ExportJobTimeline from "@/components/ExportJobTimeline.vue";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import {
  createExportClient,
  type ExportClient,
  type ExportJob,
  type ExportJobEvent,
  type ExportTarget,
} from "@/lib/exportApi";
import { allMessages, type Locale } from "@/lib/i18n";
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
const activeJob = ref<ExportJob | null>(null);
const events = ref<ExportJobEvent[]>([]);
const loadError = ref("");
const actionError = ref("");
const labels = computed(() => allMessages()[props.locale]);
const selectedPack = computed(() => props.packs.find((pack) => pack.id === selectedPackId.value) ?? null);
const selectedTarget = computed(() => targets.value.find((target) => target.id === selectedTargetId.value) ?? null);
const resultLink = computed(() => {
  const result = activeJob.value?.result;
  const link = result?.telegramUrl ?? result?.stickerSetUrl ?? result?.url;
  return typeof link === "string" ? link : "";
});

onMounted(loadTargets);
watch(() => props.patToken, loadTargets);
watch(() => props.exportClient, loadTargets);
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
    const options = JSON.parse(optionsJson.value) as Record<string, unknown>;
    activeJob.value = await exportClient().createExportJob({
      id: jobId.value.trim(),
      tenantId: props.tenantId,
      sourcePackId: selectedPackId.value,
      targetId: selectedTargetId.value,
      options,
    });
    events.value = await exportClient().listExportJobEvents(activeJob.value.id);
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
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

function exportClient() {
  return (
    props.exportClient ??
    createExportClient({
      baseUrl: import.meta.env.VITE_MSM_API_BASE_URL,
      authToken: props.patToken,
    })
  );
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
        <div class="flex flex-wrap items-center gap-3">
          <Button type="button" :aria-label="labels.queueExportJob" @click="queueExportJob">
            {{ labels.queueExportJob }}
          </Button>
          <Button type="button" variant="outline" :aria-label="labels.refreshExportJob" @click="refreshExportJob">
            {{ labels.refreshExportJob }}
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
