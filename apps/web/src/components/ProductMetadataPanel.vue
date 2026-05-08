<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createProductMetadataClient,
  type ProductMetadataClient,
  type ProductMetadataFolder,
  type ProductMetadataSubscriptionGroup,
  type ProductMetadataTag,
  type WritablePackVisibility,
} from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";

const props = defineProps<{
  locale: Locale;
  tenantId: string;
  ownerUserId: string;
  patToken?: string;
  metadataClient?: ProductMetadataClient;
}>();

const folders = ref<ProductMetadataFolder[]>([]);
const tags = ref<ProductMetadataTag[]>([]);
const subscriptionGroups = ref<ProductMetadataSubscriptionGroup[]>([]);
const loadingError = ref("");
const actionError = ref("");
const folderId = ref("folder_favorites");
const folderName = ref("Favorites");
const tagId = ref("tag_reaction");
const tagName = ref("reaction");
const subscriptionGroupId = ref("sub_weekly");
const subscriptionGroupTitle = ref("Weekly sync");
const subscriptionGroupVisibility = ref<WritablePackVisibility>("private");

const labels = computed(() => allMessages()[props.locale]);

onMounted(loadMetadata);
watch(() => props.metadataClient, loadMetadata);
watch(() => props.patToken, loadMetadata);
watch(() => [props.tenantId, props.ownerUserId], loadMetadata);

async function loadMetadata() {
  loadingError.value = "";
  try {
    const client = metadataClient();
    const [nextFolders, nextTags, nextGroups] = await Promise.all([
      client.listFolders(props.tenantId, props.ownerUserId),
      client.listTags(props.tenantId),
      client.listSubscriptionGroups(props.tenantId, props.ownerUserId),
    ]);
    folders.value = nextFolders;
    tags.value = nextTags;
    subscriptionGroups.value = nextGroups;
  } catch (error) {
    loadingError.value = error instanceof Error ? error.message : String(error);
  }
}

async function createFolder() {
  actionError.value = "";
  try {
    await metadataClient().createFolder({
      id: folderId.value.trim(),
      tenantId: props.tenantId,
      ownerUserId: props.ownerUserId,
      name: folderName.value.trim(),
    });
    await loadMetadata();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function createTag() {
  actionError.value = "";
  try {
    await metadataClient().createTag({
      id: tagId.value.trim(),
      tenantId: props.tenantId,
      name: tagName.value.trim(),
    });
    await loadMetadata();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function createSubscriptionGroup() {
  actionError.value = "";
  try {
    await metadataClient().createSubscriptionGroup({
      id: subscriptionGroupId.value.trim(),
      tenantId: props.tenantId,
      ownerUserId: props.ownerUserId,
      title: subscriptionGroupTitle.value.trim(),
      visibility: subscriptionGroupVisibility.value,
    });
    await loadMetadata();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

function metadataClient() {
  return (
    props.metadataClient ??
    createProductMetadataClient({
      baseUrl: import.meta.env.VITE_MSM_API_BASE_URL,
      authToken: props.patToken,
    })
  );
}
</script>

<template>
  <section class="flex flex-col gap-5" data-testid="product-metadata-section">
    <div class="rounded-[1.6rem] border bg-card/88 px-5 py-5">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 class="text-xl font-semibold tracking-tight">{{ labels.productMetadata }}</h2>
          <p class="mt-1 max-w-3xl text-sm leading-6 text-muted-foreground">{{ labels.productMetadataHelp }}</p>
        </div>
        <Button type="button" variant="outline" @click="loadMetadata">{{ labels.refreshTokens }}</Button>
      </div>
      <p v-if="loadingError || actionError" class="mt-4 rounded-2xl border bg-background/70 px-4 py-3 text-sm text-muted-foreground">
        {{ loadingError || actionError }}
      </p>
    </div>

    <div class="grid gap-4 2xl:grid-cols-3">
      <section class="overflow-hidden rounded-[1.35rem] border bg-card/80">
        <div class="border-b px-4 py-4">
          <h3 class="font-semibold">{{ labels.folders }}</h3>
          <p class="mt-1 text-sm text-muted-foreground">{{ folders.length }} {{ labels.folders }}</p>
        </div>
        <form class="grid gap-3 px-4 py-4" @submit.prevent>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.folderId }}
            <input v-model="folderId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.folderId" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.folderName }}
            <input v-model="folderName" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.folderName" />
          </label>
          <Button type="button" class="w-fit" :aria-label="labels.createFolder" @click="createFolder">{{ labels.createFolder }}</Button>
        </form>
        <div class="divide-y border-t">
          <article v-for="folder in folders" :key="folder.id" class="px-4 py-3">
            <p class="font-semibold">{{ folder.name }}</p>
            <p class="mt-1 text-xs text-muted-foreground">{{ folder.id }}</p>
          </article>
          <p v-if="folders.length === 0" class="px-4 py-3 text-sm text-muted-foreground">{{ labels.noFolders }}</p>
        </div>
      </section>

      <section class="overflow-hidden rounded-[1.35rem] border bg-card/80">
        <div class="border-b px-4 py-4">
          <h3 class="font-semibold">{{ labels.tags }}</h3>
          <p class="mt-1 text-sm text-muted-foreground">{{ tags.length }} {{ labels.tags }}</p>
        </div>
        <form class="grid gap-3 px-4 py-4" @submit.prevent>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.tagId }}
            <input v-model="tagId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.tagId" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.tagName }}
            <input v-model="tagName" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.tagName" />
          </label>
          <Button type="button" class="w-fit" :aria-label="labels.createTag" @click="createTag">{{ labels.createTag }}</Button>
        </form>
        <div class="flex flex-wrap gap-2 border-t px-4 py-4">
          <Badge v-for="tag in tags" :key="tag.id" variant="secondary">{{ tag.name }}</Badge>
          <p v-if="tags.length === 0" class="text-sm text-muted-foreground">{{ labels.noTags }}</p>
        </div>
      </section>

      <section class="overflow-hidden rounded-[1.35rem] border bg-card/80">
        <div class="border-b px-4 py-4">
          <h3 class="font-semibold">{{ labels.subscriptionGroups }}</h3>
          <p class="mt-1 text-sm text-muted-foreground">{{ subscriptionGroups.length }} {{ labels.subscriptionGroups }}</p>
        </div>
        <form class="grid gap-3 px-4 py-4" @submit.prevent>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.subscriptionGroupId }}
            <input v-model="subscriptionGroupId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionGroupId" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.subscriptionGroupTitle }}
            <input v-model="subscriptionGroupTitle" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionGroupTitle" />
          </label>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.subscriptionGroupVisibility }}
            <select v-model="subscriptionGroupVisibility" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionGroupVisibility">
              <option value="public">{{ labels.public }}</option>
              <option value="private">{{ labels.private }}</option>
            </select>
          </label>
          <Button type="button" class="w-fit" :aria-label="labels.createSubscriptionGroup" @click="createSubscriptionGroup">{{ labels.createSubscriptionGroup }}</Button>
        </form>
        <div class="divide-y border-t">
          <article v-for="group in subscriptionGroups" :key="group.id" class="px-4 py-3">
            <div class="flex flex-wrap items-center gap-2">
              <p class="font-semibold">{{ group.title }}</p>
              <Badge :variant="group.visibility === 'public' ? 'accent' : 'muted'">
                {{ group.visibility === "public" ? labels.public : labels.private }}
              </Badge>
            </div>
            <p class="mt-1 text-xs text-muted-foreground">{{ group.id }}</p>
          </article>
          <p v-if="subscriptionGroups.length === 0" class="px-4 py-3 text-sm text-muted-foreground">{{ labels.noSubscriptionGroups }}</p>
        </div>
      </section>
    </div>
  </section>
</template>
