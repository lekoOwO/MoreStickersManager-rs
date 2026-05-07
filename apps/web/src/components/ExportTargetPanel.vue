<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { createExportClient, type ExportClient, type ExportTarget, type ExportTargetKind } from "@/lib/exportApi";
import { allMessages, type Locale } from "@/lib/i18n";

const props = defineProps<{
  locale: Locale;
  tenantId: string;
  patToken?: string;
  exportClient?: ExportClient;
}>();

const targetKinds = ref<ExportTargetKind[]>([]);
const targets = ref<ExportTarget[]>([]);
const loadError = ref("");
const createError = ref("");
const targetId = ref("target_telegram");
const targetKind = ref("telegram");
const targetName = ref("Telegram");
const targetConfigJson = ref('{"botUsername":"msm_bot","botToken":"123:token"}');
const targetEnabled = ref(true);
const labels = computed(() => allMessages()[props.locale]);

onMounted(loadTargets);
watch(() => props.patToken, loadTargets);
watch(() => props.exportClient, loadTargets);

async function loadTargets() {
  loadError.value = "";
  try {
    const client = exportClient();
    const [kinds, configuredTargets] = await Promise.all([
      client.listExportTargetKinds(),
      client.listExportTargets(props.tenantId),
    ]);
    targetKinds.value = kinds;
    targets.value = configuredTargets;
    targetKind.value = targetKinds.value[0]?.kind ?? targetKind.value;
  } catch (error) {
    targetKinds.value = [];
    targets.value = [];
    loadError.value = error instanceof Error ? error.message : String(error);
  }
}

async function createTarget() {
  createError.value = "";
  try {
    const config = JSON.parse(targetConfigJson.value) as Record<string, unknown>;
    if (targetKind.value === "telegram" && !isValidTelegramBotToken(config.botToken)) {
      createError.value = labels.value.invalidTelegramToken;
      return;
    }
    await exportClient().createExportTarget({
      id: targetId.value.trim(),
      tenantId: props.tenantId,
      kind: targetKind.value,
      name: targetName.value.trim(),
      config,
      isEnabled: targetEnabled.value,
    });
    await loadTargets();
  } catch (error) {
    createError.value = error instanceof Error ? error.message : String(error);
  }
}

function isValidTelegramBotToken(value: unknown) {
  return typeof value === "string" && /^\d+:[A-Za-z0-9_-]+$/.test(value);
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
  <Card>
    <CardHeader>
      <CardTitle>{{ labels.exportTargets }}</CardTitle>
      <CardDescription>{{ labels.exportTargetsHelp }}</CardDescription>
    </CardHeader>
    <CardContent class="flex flex-col gap-5">
      <p v-if="loadError" class="rounded-lg border bg-background/70 p-3 text-sm text-muted-foreground">
        {{ loadError }}
      </p>

      <section class="grid gap-3 md:grid-cols-2">
        <article
          v-for="kind in targetKinds"
          :key="kind.kind"
          class="rounded-lg border bg-background/70 p-3 text-sm"
        >
          <div class="flex flex-wrap items-center gap-2">
            <p class="font-semibold">{{ kind.displayName }}</p>
            <Badge variant="outline">{{ kind.kind }}</Badge>
          </div>
          <p class="mt-2 text-muted-foreground">
            {{ kind.supportsMediaConversion ? labels.supportsConversion : labels.directExport }}
            -
            {{ kind.supportsRemotePublication ? labels.remotePublication : labels.localExport }}
          </p>
        </article>
      </section>

      <form class="grid gap-3 rounded-xl border bg-card/60 p-4" @submit.prevent="createTarget">
        <div class="grid gap-3 md:grid-cols-2">
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.targetId }}
            <input
              v-model="targetId"
              class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.targetId"
            />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.targetName }}
            <input
              v-model="targetName"
              class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.targetName"
            />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.targetKind }}
            <select
              v-model="targetKind"
              class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.targetKind"
            >
              <option v-for="kind in targetKinds" :key="kind.kind" :value="kind.kind">
                {{ kind.displayName }}
              </option>
            </select>
          </label>
          <label class="flex items-center gap-2 text-sm font-medium">
            <input v-model="targetEnabled" type="checkbox" :aria-label="labels.targetEnabled" />
            {{ labels.targetEnabled }}
          </label>
        </div>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.targetConfigJson }}
          <textarea
            v-model="targetConfigJson"
            class="min-h-28 rounded-md border bg-background px-3 py-2 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.targetConfigJson"
          />
        </label>
        <div class="flex flex-wrap items-center gap-3">
          <Button type="button" :aria-label="labels.createExportTarget" @click="createTarget">
            {{ labels.createExportTarget }}
          </Button>
          <p v-if="createError" class="text-sm text-muted-foreground">{{ createError }}</p>
        </div>
      </form>

      <p v-if="targets.length === 0" class="rounded-lg border bg-background/70 p-3 text-sm text-muted-foreground">
        {{ labels.noExportTargets }}
      </p>
      <article
        v-for="target in targets"
        :key="target.id"
        class="grid gap-3 rounded-lg border bg-background/70 p-3 text-sm md:grid-cols-[1fr_auto]"
      >
        <div class="min-w-0">
          <div class="flex flex-wrap items-center gap-2">
            <p class="font-semibold">{{ target.name }}</p>
            <Badge variant="outline">{{ target.kind }}</Badge>
            <Badge :variant="target.isEnabled ? 'secondary' : 'muted'">
              {{ target.isEnabled ? labels.enabled : labels.disabled }}
            </Badge>
          </div>
          <p class="mt-2 truncate font-mono text-xs text-muted-foreground">
            {{ JSON.stringify(target.config) }}
          </p>
        </div>
        <Badge variant="secondary">{{ target.updatedAt.split("T")[0] }}</Badge>
      </article>
    </CardContent>
  </Card>
</template>
