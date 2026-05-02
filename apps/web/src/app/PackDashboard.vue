<script setup lang="ts">
import { computed, onMounted, ref } from "vue";

import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { createPackClient } from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import { type PackVisibility, type StickerPackSummary } from "@/lib/sticker-packs";

const props = defineProps<{
  locale: Locale;
}>();

const packClient = createPackClient({
  baseUrl: import.meta.env.VITE_MSM_API_BASE_URL,
  userId: import.meta.env.VITE_MSM_USER_ID,
});
const packs = ref<StickerPackSummary[]>([]);

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

onMounted(async () => {
  packs.value = await packClient.listStickerPacks();
});

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
          <article
            v-for="pack in packs"
            :key="pack.id"
            class="grid gap-3 rounded-xl border bg-background/80 p-4 md:grid-cols-[1fr_auto] md:items-center"
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
            </div>
            <Badge :variant="pack.subscriptionReady ? 'accent' : 'muted'">
              {{ labels.subscriptionReady }}
            </Badge>
          </article>
        </CardContent>
      </Card>
    </section>
  </div>
</template>
