<script setup lang="ts">
import { computed, ref } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createProviderImportClient,
  type ProviderImportClient,
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
const errorMessage = ref("");
const isPlanning = ref(false);

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
          <p v-if="errorMessage" class="text-sm text-muted-foreground">{{ errorMessage }}</p>
        </div>
      </form>

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
    </div>
  </section>
</template>
