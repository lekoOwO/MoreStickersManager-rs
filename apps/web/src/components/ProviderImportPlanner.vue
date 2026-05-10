<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createProviderImportClient,
  type ProviderConfigResponse,
  type ProviderImportClient,
  type ProviderImportJob,
  type ProviderImportJobEvent,
  type ProviderImportPlan,
  type ProviderImportSource,
} from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";

const props = defineProps<{
  locale: Locale;
  patToken?: string;
  tenantId: string;
  ownerUserId: string;
  providerImportClient?: ProviderImportClient;
}>();

const providerId = ref<ProviderImportSource>("telegram");
const remoteId = ref("kawaii_animals");
const baseUrl = ref("");
const plan = ref<ProviderImportPlan | null>(null);
const jobId = ref("provider_job_line_1");
const targetPackId = ref("");
const job = ref<ProviderImportJob | null>(null);
const jobEvents = ref<ProviderImportJobEvent[]>([]);
const errorMessage = ref("");
const isPlanning = ref(false);
const isCreatingJob = ref(false);
const isRefreshingJob = ref(false);
const providerConfigs = ref<ProviderConfigResponse[]>([]);
const configId = ref("provider_telegram");
const configProviderId = ref<ProviderImportSource>("telegram");
const configName = ref("Telegram Import Bot");
const configJson = ref('{\n  "botToken": "123456:secret",\n  "apiBaseUrl": "https://api.telegram.org"\n}');
const configEnabled = ref(true);
const isLoadingConfigs = ref(false);
const isSavingConfig = ref(false);
const deletingConfigId = ref("");

const labels = computed(() => allMessages()[props.locale]);
const strategyLabel = computed(() => {
  if (!plan.value) {
    return "";
  }
  return plan.value.assetStrategy === "telegramBotFileApi"
    ? labels.value.providerImportTelegramBotFileApi
    : labels.value.providerImportDirectRemoteUrls;
});

async function createPlan() {
  errorMessage.value = "";
  plan.value = null;
  isPlanning.value = true;
  try {
    plan.value = await providerImportClient().createProviderImportPlan({
      tenantId: props.tenantId,
      ownerUserId: props.ownerUserId,
      providerId: providerId.value,
      remoteId: remoteId.value.trim(),
      baseUrl: baseUrl.value.trim() || null,
    });
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isPlanning.value = false;
  }
}

async function createJob() {
  errorMessage.value = "";
  job.value = null;
  jobEvents.value = [];
  isCreatingJob.value = true;
  try {
    const created = await providerImportClient().createProviderImportJob({
      id: jobId.value.trim(),
      tenantId: props.tenantId,
      ownerUserId: props.ownerUserId,
      providerId: providerId.value,
      remoteId: remoteId.value.trim(),
      targetPackId: targetPackId.value.trim() || null,
      baseUrl: baseUrl.value.trim() || null,
    });
    job.value = created;
    await refreshJob(created.id);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isCreatingJob.value = false;
  }
}

async function refreshJob(id = jobId.value.trim()) {
  if (!id) {
    return;
  }
  errorMessage.value = "";
  isRefreshingJob.value = true;
  try {
    const client = providerImportClient();
    const [refreshed, events] = await Promise.all([
      client.getProviderImportJob(id),
      client.listProviderImportJobEvents(id),
    ]);
    job.value = refreshed;
    jobEvents.value = events;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isRefreshingJob.value = false;
  }
}


async function loadProviderConfigs() {
  errorMessage.value = "";
  isLoadingConfigs.value = true;
  try {
    providerConfigs.value = await providerImportClient().listProviderConfigs(props.tenantId);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoadingConfigs.value = false;
  }
}

async function saveProviderConfig() {
  errorMessage.value = "";
  isSavingConfig.value = true;
  try {
    const config = JSON.parse(configJson.value) as Record<string, unknown>;
    const saved = await providerImportClient().upsertProviderConfig(configId.value.trim(), {
      tenantId: props.tenantId,
      providerId: configProviderId.value,
      name: configName.value.trim(),
      config,
      isEnabled: configEnabled.value,
    });
    providerConfigs.value = [saved, ...providerConfigs.value.filter((config) => config.id !== saved.id)];
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSavingConfig.value = false;
  }
}

async function deleteProviderConfig(id: string) {
  errorMessage.value = "";
  deletingConfigId.value = id;
  try {
    await providerImportClient().deleteProviderConfig(id);
    providerConfigs.value = providerConfigs.value.filter((config) => config.id !== id);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    deletingConfigId.value = "";
  }
}

function formatConfig(config: Record<string, unknown>) {
  return JSON.stringify(config, null, 2);
}

onMounted(loadProviderConfigs);
watch(
  () => [props.tenantId, props.providerImportClient, props.patToken] as const,
  () => {
    void loadProviderConfigs();
  },
);
function providerImportClient() {
  return (
    props.providerImportClient ??
    createProviderImportClient({
      baseUrl: import.meta.env.VITE_MSM_API_BASE_URL,
      authToken: props.patToken,
    })
  );
}
</script>

<template>
  <section class="overflow-hidden rounded-[1.6rem] border bg-card/88" data-testid="provider-import-planner">
    <div class="flex flex-wrap items-start justify-between gap-3 border-b px-5 py-4">
      <div>
        <h2 class="text-lg font-semibold">{{ labels.providerImportPlanner }}</h2>
        <p class="mt-1 max-w-3xl text-sm text-muted-foreground">{{ labels.providerImportPlannerHelp }}</p>
      </div>
      <Badge variant="secondary">{{ labels.providerImportPlanningOnly }}</Badge>
    </div>

    <div class="grid gap-5 p-5 xl:grid-cols-[0.8fr_1.2fr]">
      <form class="grid gap-4" @submit.prevent="createPlan">
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.provider }}
          <select
            v-model="providerId"
            class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.provider"
          >
            <option value="telegram">Telegram</option>
            <option value="line-stickers">LINE Stickers</option>
          </select>
        </label>

        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.providerRemoteId }}
          <input
            v-model="remoteId"
            class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.providerRemoteId"
            autocomplete="off"
          />
          <span class="text-xs text-muted-foreground">{{ labels.providerRemoteIdHelp }}</span>
        </label>

        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.providerBaseUrl }}
          <input
            v-model="baseUrl"
            class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.providerBaseUrl"
            autocomplete="off"
            placeholder="https://api.telegram.org"
          />
          <span class="text-xs text-muted-foreground">{{ labels.providerBaseUrlHelp }}</span>
        </label>

        <div class="flex flex-wrap items-center gap-3">
          <Button type="submit" :disabled="isPlanning || !remoteId.trim()" :aria-label="labels.createProviderImportPlan">
            {{ isPlanning ? labels.loading : labels.createProviderImportPlan }}
          </Button>
          <Button
            type="button"
            variant="outline"
            :disabled="isCreatingJob || !remoteId.trim() || !jobId.trim()"
            :aria-label="labels.queueProviderImportJob"
            @click="createJob"
          >
            {{ isCreatingJob ? labels.loading : labels.queueProviderImportJob }}
          </Button>
        </div>

        <div class="grid gap-3 rounded-[1.35rem] border bg-background/70 p-4">
          <div>
            <h3 class="text-sm font-semibold">{{ labels.providerImportJobControls }}</h3>
            <p class="mt-1 text-xs text-muted-foreground">{{ labels.providerImportJobControlsHelp }}</p>
          </div>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.providerImportJobId }}
            <input
              v-model="jobId"
              class="h-10 rounded-lg border bg-background px-3 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.providerImportJobId"
              autocomplete="off"
            />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.providerImportTargetPackId }}
            <input
              v-model="targetPackId"
              class="h-10 rounded-lg border bg-background px-3 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.providerImportTargetPackId"
              autocomplete="off"
            />
            <span class="text-xs text-muted-foreground">{{ labels.providerImportTargetPackIdHelp }}</span>
          </label>
          <Button
            type="button"
            variant="secondary"
            :disabled="isRefreshingJob || !jobId.trim()"
            :aria-label="labels.refreshProviderImportJob"
            @click="refreshJob()"
          >
            {{ isRefreshingJob ? labels.loading : labels.refreshProviderImportJob }}
          </Button>
        </div>

        <p v-if="errorMessage" class="rounded-2xl border bg-muted/40 p-3 text-sm text-muted-foreground">
          {{ errorMessage }}
        </p>
      </form>

      <section class="grid gap-4 rounded-[1.35rem] border bg-background/70 p-4 xl:col-span-2" data-testid="provider-configs-panel">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h3 class="text-sm font-semibold">{{ labels.providerConfigs }}</h3>
            <p class="mt-1 text-xs text-muted-foreground">{{ labels.providerConfigsHelp }}</p>
          </div>
          <Button
            type="button"
            variant="secondary"
            :disabled="isLoadingConfigs"
            :aria-label="labels.refreshProviderConfigs"
            @click="loadProviderConfigs"
          >
            {{ isLoadingConfigs ? labels.loading : labels.refreshProviderConfigs }}
          </Button>
        </div>

        <form class="grid gap-3 lg:grid-cols-[minmax(9rem,0.7fr)_minmax(9rem,0.7fr)_minmax(10rem,1fr)_auto]" @submit.prevent="saveProviderConfig">
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.providerConfigId }}
            <input
              v-model="configId"
              class="h-10 rounded-lg border bg-background px-3 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
              aria-label="Provider config ID"
              autocomplete="off"
            />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.providerConfigSource }}
            <select
              v-model="configProviderId"
              class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              aria-label="Provider config source"
            >
              <option value="telegram">Telegram</option>
              <option value="line-stickers">LINE Stickers</option>
            </select>
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.providerConfigName }}
            <input
              v-model="configName"
              class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              aria-label="Provider config name"
              autocomplete="off"
            />
          </label>
          <label class="flex items-end gap-2 pb-2 text-sm font-medium lg:justify-end">
            <input v-model="configEnabled" type="checkbox" class="size-4 rounded border" :aria-label="labels.providerConfigEnabled" />
            {{ labels.providerConfigEnabled }}
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium lg:col-span-3">
            {{ labels.providerConfigJson }}
            <textarea
              v-model="configJson"
              class="min-h-32 rounded-lg border bg-background px-3 py-2 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
              aria-label="Provider config JSON"
              spellcheck="false"
            />
            <span class="text-xs text-muted-foreground">{{ labels.providerConfigJsonHelp }}</span>
          </label>
          <div class="flex items-end">
            <Button type="submit" class="w-full" :disabled="isSavingConfig || !configId.trim() || !configName.trim()" aria-label="Save provider config" @click="saveProviderConfig">
              {{ isSavingConfig ? labels.loading : labels.saveProviderConfig }}
            </Button>
          </div>
        </form>

        <div v-if="providerConfigs.length" class="grid gap-3">
          <article
            v-for="config in providerConfigs"
            :key="config.id"
            class="grid gap-3 rounded-2xl border bg-card/70 p-3 lg:grid-cols-[minmax(12rem,0.9fr)_minmax(16rem,1.4fr)_auto]"
          >
            <div>
              <div class="flex flex-wrap items-center gap-2">
                <h4 class="text-sm font-semibold">{{ config.name }}</h4>
                <Badge :variant="config.isEnabled ? 'secondary' : 'outline'">
                  {{ config.isEnabled ? labels.providerConfigStatusEnabled : labels.providerConfigStatusDisabled }}
                </Badge>
              </div>
              <p class="mt-1 break-all font-mono text-xs text-muted-foreground">{{ config.id }} · {{ config.providerId }}</p>
            </div>
            <pre class="max-h-40 overflow-auto rounded-xl bg-muted/60 p-3 text-xs text-muted-foreground">{{ formatConfig(config.config) }}</pre>
            <div class="flex items-start lg:justify-end">
              <Button
                type="button"
                variant="outline"
                :disabled="deletingConfigId === config.id"
                :aria-label="'Delete provider config ' + config.id"
                @click="deleteProviderConfig(config.id)"
              >
                {{ deletingConfigId === config.id ? labels.loading : labels.deleteProviderConfig }}
              </Button>
            </div>
          </article>
        </div>
        <p v-else class="rounded-2xl border border-dashed p-4 text-sm text-muted-foreground">
          {{ labels.providerConfigsEmpty }}
        </p>
      </section>

      <div class="grid gap-5">
      <div class="rounded-[1.35rem] border bg-background/70">
        <div class="border-b px-4 py-3">
          <h3 class="text-sm font-semibold">{{ labels.providerImportPlanResult }}</h3>
          <p class="mt-1 text-xs text-muted-foreground">{{ labels.providerImportPlanResultHelp }}</p>
        </div>
        <div v-if="plan" class="grid gap-3 p-4">
          <div class="grid gap-3 md:grid-cols-2">
            <div class="rounded-2xl bg-muted/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.provider }}</p>
              <p class="mt-2 font-mono text-sm">{{ plan.providerId }}</p>
            </div>
            <div class="rounded-2xl bg-muted/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.providerAssetStrategy }}</p>
              <p class="mt-2 text-sm font-medium">{{ strategyLabel }}</p>
            </div>
          </div>
          <div class="rounded-2xl bg-muted/60 p-3">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.providerMetadataRequest }}</p>
            <p class="mt-2 break-all font-mono text-xs">{{ plan.metadataRequest.method }} {{ plan.metadataRequest.url }}</p>
          </div>
          <div class="rounded-2xl bg-muted/60 p-3">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.providerRedactedHeaders }}</p>
            <p v-if="plan.metadataRequest.redactedHeaders.length === 0" class="mt-2 text-sm text-muted-foreground">
              {{ labels.providerNoRedactedHeaders }}
            </p>
            <ul v-else class="mt-2 grid gap-1">
              <li
                v-for="header in plan.metadataRequest.redactedHeaders"
                :key="`${header.name}:${header.value}`"
                class="break-all font-mono text-xs text-muted-foreground"
              >
                {{ header.name }}: {{ header.value }}
              </li>
            </ul>
          </div>
        </div>
        <div v-else class="p-4 text-sm text-muted-foreground">
          {{ labels.providerImportPlanEmpty }}
        </div>
      </div>

      <div class="rounded-[1.35rem] border bg-background/70" data-testid="provider-import-job-panel">
        <div class="flex flex-wrap items-start justify-between gap-3 border-b px-4 py-3">
          <div>
            <h3 class="text-sm font-semibold">{{ labels.providerImportJobStatus }}</h3>
            <p class="mt-1 text-xs text-muted-foreground">{{ labels.providerImportJobStatusHelp }}</p>
          </div>
          <Badge v-if="job" variant="secondary">{{ job.status }}</Badge>
        </div>
        <div v-if="job" class="grid gap-4 p-4">
          <div class="grid gap-3 md:grid-cols-3">
            <div class="rounded-2xl bg-muted/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.providerImportJobId }}</p>
              <p class="mt-2 break-all font-mono text-xs">{{ job.id }}</p>
            </div>
            <div class="rounded-2xl bg-muted/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.provider }}</p>
              <p class="mt-2 font-mono text-sm">{{ job.providerId }}</p>
            </div>
            <div class="rounded-2xl bg-muted/60 p-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ labels.providerImportAttempts }}</p>
              <p class="mt-2 font-mono text-sm">{{ job.attemptCount }} / {{ job.maxAttempts }}</p>
            </div>
          </div>
          <div v-if="job.errorSummary" class="rounded-2xl bg-muted/60 p-3 text-sm text-muted-foreground">
            {{ job.errorSummary }}
          </div>
          <div>
            <h4 class="text-sm font-semibold">{{ labels.providerImportJobEvents }}</h4>
            <ol v-if="jobEvents.length" class="mt-3 grid gap-2">
              <li
                v-for="event in jobEvents"
                :key="`${event.jobId}:${event.sequence}`"
                class="grid gap-1 rounded-2xl bg-muted/60 p-3"
              >
                <div class="flex flex-wrap items-center justify-between gap-2">
                  <span class="font-mono text-xs text-muted-foreground">#{{ event.sequence }} {{ event.stage }}</span>
                  <Badge variant="outline">{{ event.level }}</Badge>
                </div>
                <p class="text-sm">{{ event.message }}</p>
              </li>
            </ol>
            <p v-else class="mt-3 text-sm text-muted-foreground">{{ labels.providerImportJobEventsEmpty }}</p>
          </div>
        </div>
        <div v-else class="p-4 text-sm text-muted-foreground">
          {{ labels.providerImportJobEmpty }}
        </div>
      </div>
      </div>
    </div>
  </section>
</template>
