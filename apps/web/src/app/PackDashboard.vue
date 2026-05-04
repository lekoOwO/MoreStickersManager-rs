<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { createPackClient, type PackClient, type WritablePackVisibility } from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import { type PackVisibility, type StickerPackSummary } from "@/lib/sticker-packs";

const props = defineProps<{
  locale: Locale;
  patToken?: string;
  packClient?: PackClient;
  tenantId?: string;
  ownerUserId?: string;
}>();

const packs = ref<StickerPackSummary[]>([]);
const loadError = ref("");
const actionError = ref("");
const importError = ref("");
const importPackId = ref("");
const importVisibility = ref<WritablePackVisibility>("private");
const importJson = ref("");
const drafts = ref<Record<string, { title: string; visibility: WritablePackVisibility }>>({});

const labels = computed(() => allMessages()[props.locale]);
const totalStickers = computed(() => packs.value.reduce((sum, pack) => sum + pack.stickerCount, 0));
const publicPackCount = computed(() => packs.value.filter((pack) => pack.visibility === "public").length);
const privatePackCount = computed(() => packs.value.filter((pack) => pack.visibility === "private").length);
const providerCounts = computed(() => {
  return packs.value.reduce<Record<string, number>>((counts, pack) => {
    counts[pack.provider] = (counts[pack.provider] ?? 0) + 1;
    return counts;
  }, {});
});

onMounted(loadPacks);
watch(() => props.patToken, loadPacks);
watch(() => props.packClient, loadPacks);

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
      tenantId: props.tenantId ?? import.meta.env.VITE_MSM_TENANT_ID ?? "tenant_1",
      ownerUserId: props.ownerUserId ?? import.meta.env.VITE_MSM_USER_ID ?? "user_1",
      packId: importPackId.value.trim(),
      visibility: importVisibility.value,
      pack,
    });
    importJson.value = "";
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
  <div class="flex flex-col gap-6">
    <section class="grid gap-4 md:grid-cols-2 xl:grid-cols-4" aria-label="Metrics">
      <Card>
        <CardHeader>
          <CardDescription>{{ labels.totalPacks }}</CardDescription>
          <CardTitle class="text-3xl">{{ packs.length }}</CardTitle>
        </CardHeader>
      </Card>
      <Card>
        <CardHeader>
          <CardDescription>{{ labels.managedStickers }}</CardDescription>
          <CardTitle class="text-3xl">{{ totalStickers }}</CardTitle>
        </CardHeader>
      </Card>
      <Card>
        <CardHeader>
          <CardDescription>{{ labels.publicPacks }}</CardDescription>
          <CardTitle class="text-3xl">{{ publicPackCount }}</CardTitle>
        </CardHeader>
      </Card>
      <Card>
        <CardHeader>
          <CardDescription>{{ labels.privatePacks }}</CardDescription>
          <CardTitle class="text-3xl">{{ privatePackCount }}</CardTitle>
        </CardHeader>
      </Card>
    </section>

    <Card>
      <CardHeader>
        <CardTitle>{{ labels.importStickerPack }}</CardTitle>
        <CardDescription>{{ labels.importStickerPackHelp }}</CardDescription>
      </CardHeader>
      <CardContent class="grid gap-4">
        <div class="grid gap-3 md:grid-cols-[1fr_12rem]">
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.importPackId }}
            <input
              v-model="importPackId"
              class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
              :aria-label="labels.importPackId"
            />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.importVisibility }}
            <select
              v-model="importVisibility"
              class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
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
            class="min-h-40 rounded-md border bg-background px-3 py-2 font-mono text-sm outline-none focus:ring-2 focus:ring-ring"
            :aria-label="labels.importPackJson"
          />
        </label>
        <div class="flex flex-wrap items-center gap-3">
          <Button type="button" :aria-label="labels.importStickerPack" @click="importPack">
            {{ labels.importStickerPack }}
          </Button>
          <p v-if="importError" class="text-sm text-muted-foreground">{{ importError }}</p>
        </div>
      </CardContent>
    </Card>

    <section class="grid gap-4 lg:grid-cols-[0.8fr_1.2fr]">
      <Card>
        <CardHeader>
          <CardTitle>{{ labels.providerCoverage }}</CardTitle>
          <CardDescription>Telegram / LINE first, more providers planned.</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-3">
          <div
            v-for="(count, provider) in providerCounts"
            :key="provider"
            class="flex items-center justify-between rounded-lg border bg-background/70 px-4 py-3"
          >
            <span class="font-medium">{{ provider }}</span>
            <Badge variant="secondary">{{ count }}</Badge>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{{ labels.recentPacks }}</CardTitle>
          <CardDescription>{{ labels.dashboardSubtitle }}</CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-3">
          <p v-if="loadError" class="rounded-lg border bg-background/70 px-4 py-3 text-sm text-muted-foreground">
            {{ loadError }}
          </p>
          <p v-if="actionError" class="rounded-lg border bg-background/70 px-4 py-3 text-sm text-muted-foreground">
            {{ actionError }}
          </p>
          <article
            v-for="pack in packs"
            :key="pack.id"
            class="grid gap-4 rounded-xl border bg-background/80 p-4 xl:grid-cols-[1fr_18rem] xl:items-start"
          >
            <div class="flex min-w-0 flex-col gap-2">
              <div class="flex flex-wrap items-center gap-2">
                <h3 class="truncate text-base font-semibold">{{ pack.title }}</h3>
                <Badge variant="outline">{{ pack.provider }}</Badge>
                <Badge :variant="visibilityVariant(pack.visibility)">
                  {{ visibilityLabel(pack.visibility) }}
                </Badge>
              </div>
              <p class="text-sm text-muted-foreground">
                {{ pack.stickerCount }} {{ labels.totalStickers }} · {{ labels.updated }} {{ pack.updatedAt }}
              </p>
              <Badge class="w-fit" :variant="pack.subscriptionReady ? 'accent' : 'muted'">
                {{ labels.subscriptionReady }}
              </Badge>
            </div>
            <form class="grid gap-3 rounded-lg border bg-card/60 p-3" @submit.prevent>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.packTitle }}
                <input
                  v-model="drafts[pack.id].title"
                  class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.packTitle"
                />
              </label>
              <label class="flex flex-col gap-2 text-sm font-medium">
                {{ labels.packVisibility }}
                <select
                  v-model="drafts[pack.id].visibility"
                  class="h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring"
                  :aria-label="labels.packVisibility"
                >
                  <option value="public">{{ labels.public }}</option>
                  <option value="private">{{ labels.private }}</option>
                </select>
              </label>
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
        </CardContent>
      </Card>
    </section>
  </div>
</template>
