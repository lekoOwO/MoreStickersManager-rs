<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  createProductMetadataClient,
  type ProductMetadataClient,
  type ProductMetadataFolder,
  type SubscriptionAccessResourceType,
  type SubscriptionAccessTokenResponse,
  type ProductMetadataSubscriptionGroup,
  type ProductMetadataTag,
  type WritablePackVisibility,
} from "@/lib/api-client";
import { allMessages, type Locale } from "@/lib/i18n";
import type { StickerPackSummary } from "@/lib/sticker-packs";

const props = defineProps<{
  locale: Locale;
  tenantId: string;
  ownerUserId: string;
  patToken?: string;
  metadataClient?: ProductMetadataClient;
  packs?: StickerPackSummary[];
}>();

const folders = ref<ProductMetadataFolder[]>([]);
const tags = ref<ProductMetadataTag[]>([]);
const subscriptionGroups = ref<ProductMetadataSubscriptionGroup[]>([]);
const subscriptionLinks = ref<SubscriptionAccessTokenResponse[]>([]);
const loadingError = ref("");
const actionError = ref("");
const createdSubscriptionSecret = ref("");
const folderId = ref("folder_favorites");
const folderName = ref("Favorites");
const tagId = ref("tag_reaction");
const tagName = ref("reaction");
const subscriptionGroupId = ref("sub_weekly");
const subscriptionGroupTitle = ref("Weekly sync");
const subscriptionGroupVisibility = ref<WritablePackVisibility>("private");
const selectedPackId = ref("");
const selectedFolderId = ref("");
const selectedTagId = ref("");
const selectedSubscriptionGroupId = ref("");
const subscriptionLinkId = ref("packlink");
const subscriptionLinkResourceType = ref<SubscriptionAccessResourceType>("pack");
const subscriptionLinkResourceId = ref("");
const folderPackIds = ref<string[]>([]);
const packTagIds = ref<string[]>([]);
const subscriptionGroupPackIds = ref<string[]>([]);

const labels = computed(() => allMessages()[props.locale]);
const packs = computed(() => props.packs ?? []);

onMounted(loadMetadata);
watch(() => props.metadataClient, loadMetadata);
watch(() => props.patToken, loadMetadata);
watch(() => [props.tenantId, props.ownerUserId], loadMetadata);
watch(() => props.packs, refreshSelections, { deep: true });
watch(selectedPackId, loadMemberships);
watch(selectedFolderId, loadFolderMemberships);
watch(selectedSubscriptionGroupId, loadSubscriptionGroupMemberships);

async function loadMetadata() {
  loadingError.value = "";
  try {
    const client = metadataClient();
    const [nextFolders, nextTags, nextGroups, nextLinks] = await Promise.all([
      client.listFolders(props.tenantId, props.ownerUserId),
      client.listTags(props.tenantId),
      client.listSubscriptionGroups(props.tenantId, props.ownerUserId),
      client.listSubscriptionLinks(props.ownerUserId),
    ]);
    folders.value = nextFolders;
    tags.value = nextTags;
    subscriptionGroups.value = nextGroups;
    subscriptionLinks.value = nextLinks;
    ensureSelections();
    await loadMemberships();
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

async function addPackToFolder() {
  actionError.value = "";
  try {
    await metadataClient().addPackToFolder({
      folderId: selectedFolderId.value,
      packId: selectedPackId.value,
      sortOrder: 0,
    });
    await loadFolderMemberships();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function removePackFromFolder() {
  actionError.value = "";
  try {
    await metadataClient().removePackFromFolder(selectedFolderId.value, selectedPackId.value);
    await loadFolderMemberships();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function addTagToPack() {
  actionError.value = "";
  try {
    await metadataClient().addTagToPack(selectedPackId.value, selectedTagId.value);
    await loadPackTagMemberships();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function removeTagFromPack() {
  actionError.value = "";
  try {
    await metadataClient().removeTagFromPack(selectedPackId.value, selectedTagId.value);
    await loadPackTagMemberships();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function addPackToSubscriptionGroup() {
  actionError.value = "";
  try {
    await metadataClient().addPackToSubscriptionGroup({
      subscriptionGroupId: selectedSubscriptionGroupId.value,
      packId: selectedPackId.value,
      sortOrder: 0,
    });
    await loadSubscriptionGroupMemberships();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function removePackFromSubscriptionGroup() {
  actionError.value = "";
  try {
    await metadataClient().removePackFromSubscriptionGroup(selectedSubscriptionGroupId.value, selectedPackId.value);
    await loadSubscriptionGroupMemberships();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function createSubscriptionLink() {
  actionError.value = "";
  createdSubscriptionSecret.value = "";
  try {
    const created = await metadataClient().createSubscriptionLink({
      id: subscriptionLinkId.value.trim(),
      resourceType: subscriptionLinkResourceType.value,
      resourceId: subscriptionLinkResourceId.value.trim(),
    });
    createdSubscriptionSecret.value = created.token;
    await loadMetadata();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function rotateSubscriptionLink(tokenId: string) {
  actionError.value = "";
  createdSubscriptionSecret.value = "";
  try {
    const rotated = await metadataClient().rotateSubscriptionLink(tokenId);
    createdSubscriptionSecret.value = rotated.token;
    await loadMetadata();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function revokeSubscriptionLink(tokenId: string) {
  actionError.value = "";
  try {
    await metadataClient().revokeSubscriptionLink(tokenId);
    await loadMetadata();
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error);
  }
}

async function refreshSelections() {
  ensureSelections();
  await loadMemberships();
}

function ensureSelections() {
  selectedPackId.value ||= packs.value[0]?.id ?? "";
  selectedFolderId.value ||= folders.value[0]?.id ?? "";
  selectedTagId.value ||= tags.value[0]?.id ?? "";
  selectedSubscriptionGroupId.value ||= subscriptionGroups.value[0]?.id ?? "";
  subscriptionLinkResourceId.value ||= selectedPackId.value || selectedSubscriptionGroupId.value;
}

async function loadMemberships() {
  await Promise.all([loadFolderMemberships(), loadPackTagMemberships(), loadSubscriptionGroupMemberships()]);
}

async function loadFolderMemberships() {
  folderPackIds.value = selectedFolderId.value ? await metadataClient().listFolderPacks(selectedFolderId.value) : [];
}

async function loadPackTagMemberships() {
  packTagIds.value = selectedPackId.value ? await metadataClient().listPackTags(selectedPackId.value) : [];
}

async function loadSubscriptionGroupMemberships() {
  subscriptionGroupPackIds.value = selectedSubscriptionGroupId.value
    ? await metadataClient().listSubscriptionGroupPacks(selectedSubscriptionGroupId.value)
    : [];
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

    <section class="overflow-hidden rounded-[1.35rem] border bg-card/80">
      <div class="border-b px-5 py-4">
        <h3 class="font-semibold">{{ labels.membershipConsole }}</h3>
        <p class="mt-1 max-w-3xl text-sm text-muted-foreground">{{ labels.membershipConsoleHelp }}</p>
      </div>

      <div class="grid gap-4 px-5 py-5 xl:grid-cols-[minmax(16rem,0.8fr)_repeat(3,minmax(0,1fr))]">
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.packToOrganize }}
          <select v-model="selectedPackId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.packToOrganize">
            <option v-for="pack in packs" :key="pack.id" :value="pack.id">{{ pack.title }} · {{ pack.id }}</option>
          </select>
          <span v-if="packs.length === 0" class="text-xs text-muted-foreground">{{ labels.noPacksToOrganize }}</span>
        </label>

        <form class="flex flex-col gap-3 rounded-2xl border bg-background/45 p-4" @submit.prevent>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.folderMembershipTarget }}
            <select v-model="selectedFolderId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.folderMembershipTarget">
              <option v-for="folder in folders" :key="folder.id" :value="folder.id">{{ folder.name }}</option>
            </select>
          </label>
          <p class="text-xs text-muted-foreground">{{ labels.currentFolderMembers }}: {{ folderPackIds.length }}</p>
          <div class="flex flex-wrap gap-2">
            <Button type="button" :disabled="!selectedPackId || !selectedFolderId" :aria-label="labels.addPackToFolder" @click="addPackToFolder">{{ labels.add }}</Button>
            <Button type="button" variant="outline" :disabled="!selectedPackId || !selectedFolderId" :aria-label="labels.removePackFromFolder" @click="removePackFromFolder">{{ labels.remove }}</Button>
          </div>
        </form>

        <form class="flex flex-col gap-3 rounded-2xl border bg-background/45 p-4" @submit.prevent>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.tagMembershipTarget }}
            <select v-model="selectedTagId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.tagMembershipTarget">
              <option v-for="tag in tags" :key="tag.id" :value="tag.id">{{ tag.name }}</option>
            </select>
          </label>
          <p class="text-xs text-muted-foreground">{{ labels.currentPackTags }}: {{ packTagIds.length }}</p>
          <div class="flex flex-wrap gap-2">
            <Button type="button" :disabled="!selectedPackId || !selectedTagId" :aria-label="labels.addTagToPack" @click="addTagToPack">{{ labels.add }}</Button>
            <Button type="button" variant="outline" :disabled="!selectedPackId || !selectedTagId" :aria-label="labels.removeTagFromPack" @click="removeTagFromPack">{{ labels.remove }}</Button>
          </div>
        </form>

        <form class="flex flex-col gap-3 rounded-2xl border bg-background/45 p-4" @submit.prevent>
          <label class="flex flex-col gap-2 text-sm font-medium">
            {{ labels.subscriptionMembershipTarget }}
            <select v-model="selectedSubscriptionGroupId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionMembershipTarget">
              <option v-for="group in subscriptionGroups" :key="group.id" :value="group.id">{{ group.title }}</option>
            </select>
          </label>
          <p class="text-xs text-muted-foreground">{{ labels.currentSubscriptionMembers }}: {{ subscriptionGroupPackIds.length }}</p>
          <div class="flex flex-wrap gap-2">
            <Button type="button" :disabled="!selectedPackId || !selectedSubscriptionGroupId" :aria-label="labels.addPackToSubscriptionGroup" @click="addPackToSubscriptionGroup">{{ labels.add }}</Button>
            <Button type="button" variant="outline" :disabled="!selectedPackId || !selectedSubscriptionGroupId" :aria-label="labels.removePackFromSubscriptionGroup" @click="removePackFromSubscriptionGroup">{{ labels.remove }}</Button>
          </div>
        </form>
      </div>
    </section>

    <section class="overflow-hidden rounded-[1.35rem] border bg-card/80">
      <div class="grid gap-4 border-b px-5 py-4 xl:grid-cols-[1fr_auto]">
        <div>
          <h3 class="font-semibold">{{ labels.subscriptionLinks }}</h3>
          <p class="mt-1 max-w-3xl text-sm text-muted-foreground">{{ labels.subscriptionLinksHelp }}</p>
        </div>
        <Badge variant="secondary">{{ subscriptionLinks.length }} {{ labels.subscriptionLinks }}</Badge>
      </div>

      <form class="grid gap-3 px-5 py-5 lg:grid-cols-[minmax(9rem,0.7fr)_minmax(10rem,0.8fr)_minmax(12rem,1fr)_auto]" @submit.prevent>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.subscriptionLinkId }}
          <input v-model="subscriptionLinkId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionLinkId" />
        </label>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.subscriptionLinkResourceType }}
          <select v-model="subscriptionLinkResourceType" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionLinkResourceType">
            <option value="pack">{{ labels.packLink }}</option>
            <option value="subscriptionGroup">{{ labels.subscriptionGroupLink }}</option>
          </select>
        </label>
        <label class="flex flex-col gap-2 text-sm font-medium">
          {{ labels.subscriptionLinkResourceId }}
          <input v-model="subscriptionLinkResourceId" class="h-10 rounded-lg border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-ring" :aria-label="labels.subscriptionLinkResourceId" />
        </label>
        <Button type="button" class="self-end" :aria-label="labels.createSubscriptionLink" @click="createSubscriptionLink">{{ labels.createSubscriptionLink }}</Button>
      </form>

      <p v-if="createdSubscriptionSecret" class="mx-5 mb-4 rounded-2xl border bg-background/70 px-4 py-3 font-mono text-xs text-muted-foreground" data-testid="subscription-secret">
        {{ labels.subscriptionLinkSecretOnce }} {{ createdSubscriptionSecret }}
      </p>

      <div class="divide-y border-t">
        <article v-for="link in subscriptionLinks" :key="link.id" class="grid gap-3 px-5 py-4 lg:grid-cols-[1fr_auto]">
          <div>
            <div class="flex flex-wrap items-center gap-2">
              <p class="font-semibold">{{ link.id }}</p>
              <Badge :variant="link.revokedAt ? 'muted' : 'secondary'">{{ link.revokedAt ? labels.revoked : labels.active }}</Badge>
              <Badge variant="outline">{{ link.resourceType === "pack" ? labels.packLink : labels.subscriptionGroupLink }}</Badge>
            </div>
            <p class="mt-1 font-mono text-xs text-muted-foreground">{{ link.resourceId }}</p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Button type="button" variant="outline" :aria-label="`${labels.rotateSubscriptionLink} ${link.id}`" @click="rotateSubscriptionLink(link.id)">{{ labels.rotateSubscriptionLink }}</Button>
            <Button type="button" variant="outline" :aria-label="`${labels.revokeSubscriptionLink} ${link.id}`" @click="revokeSubscriptionLink(link.id)">{{ labels.revokeSubscriptionLink }}</Button>
          </div>
        </article>
        <p v-if="subscriptionLinks.length === 0" class="px-5 py-4 text-sm text-muted-foreground">{{ labels.noSubscriptionLinks }}</p>
      </div>
    </section>
  </section>
</template>
