<script setup lang="ts">
import { computed, ref } from "vue";

import { Button } from "@/components/ui/button";
import { createPortabilityClient, type PortabilityClient } from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import { resolveApiBaseUrl } from "@/lib/runtime-config";

const props = defineProps<{
  locale: Locale;
  patToken?: string;
  tenantId: string;
  ownerUserId: string;
  portabilityClient?: PortabilityClient;
}>();

const labels = computed(() => allMessages()[props.locale]);
const targetTenantId = ref(props.tenantId);
const portableJson = ref("");
const status = ref("");
const error = ref("");
const isExporting = ref(false);
const isImporting = ref(false);

async function exportUserData() {
  error.value = "";
  status.value = "";
  isExporting.value = true;
  try {
    const exported = await client().exportUserData(props.ownerUserId);
    portableJson.value = JSON.stringify(exported, null, 2);
    status.value = labels.value.portabilityExported;
  } catch (nextError) {
    error.value = nextError instanceof Error ? nextError.message : String(nextError);
  } finally {
    isExporting.value = false;
  }
}

async function importUserData() {
  error.value = "";
  status.value = "";
  isImporting.value = true;
  try {
    await client().importUserData({
      tenantId: targetTenantId.value.trim(),
      export: JSON.parse(portableJson.value),
    });
    status.value = labels.value.portabilityImported;
  } catch (nextError) {
    error.value = nextError instanceof Error ? nextError.message : String(nextError);
  } finally {
    isImporting.value = false;
  }
}

function client() {
  return (
    props.portabilityClient ??
    createPortabilityClient({
      baseUrl: resolveApiBaseUrl(),
      authToken: props.patToken,
    })
  );
}
</script>

<template>
  <section class="rounded-[1.6rem] border bg-card/88 p-5" data-testid="portability-panel">
    <div class="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
      <div>
        <h2 class="text-lg font-semibold">{{ labels.portability }}</h2>
        <p class="mt-1 max-w-3xl text-sm text-muted-foreground">{{ labels.portabilityHelp }}</p>
      </div>
      <Button type="button" :disabled="isExporting" @click="exportUserData">{{ labels.portabilityExport }}</Button>
    </div>

    <div class="mt-5 grid gap-5 xl:grid-cols-[minmax(0,1fr)_22rem]">
      <label class="flex flex-col gap-2">
        <span class="text-sm font-medium">{{ labels.portabilityPayload }}</span>
        <textarea
          v-model="portableJson"
          class="min-h-80 rounded-2xl border bg-background px-4 py-3 font-mono text-sm outline-none transition focus:border-primary"
          :aria-label="labels.portabilityPayload"
        />
        <span class="text-xs text-muted-foreground">{{ labels.portabilityExportHelp }}</span>
      </label>

      <aside class="flex flex-col gap-4 rounded-3xl border bg-background/70 p-4">
        <div>
          <h3 class="text-sm font-semibold">{{ labels.portabilityImport }}</h3>
          <p class="mt-1 text-xs text-muted-foreground">{{ labels.portabilityImportHelp }}</p>
        </div>
        <label class="flex flex-col gap-2">
          <span class="text-sm font-medium">{{ labels.portabilityTargetTenant }}</span>
          <input
            v-model="targetTenantId"
            class="rounded-xl border bg-background px-3 py-2 text-sm outline-none transition focus:border-primary"
            :aria-label="labels.portabilityTargetTenant"
          />
        </label>
        <Button type="button" :disabled="isImporting || !portableJson.trim() || !targetTenantId.trim()" @click="importUserData">
          {{ labels.portabilityImport }}
        </Button>
        <p v-if="status" class="rounded-2xl border border-primary/30 bg-primary/10 px-3 py-2 text-sm text-primary">{{ status }}</p>
        <p v-if="error" class="rounded-2xl border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">{{ error }}</p>
      </aside>
    </div>
  </section>
</template>
