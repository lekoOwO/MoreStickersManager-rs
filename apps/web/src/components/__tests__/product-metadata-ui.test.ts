import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import ProductMetadataPanel from "@/components/ProductMetadataPanel.vue";
import type { ProductMetadataClient } from "@/lib/api-client";
import type { StickerPackSummary } from "@/lib/sticker-packs";

describe("product metadata UI", () => {
  it("creates and lists folders, tags, and subscription groups", async () => {
    const client: ProductMetadataClient = {
      listFolders: vi.fn(async () => [
        {
          id: "folder_1",
          tenantId: "tenant_1",
          ownerUserId: "user_1",
          name: "Favorites",
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
      createFolder: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
      listFolderPacks: vi.fn(async () => []),
      addPackToFolder: vi.fn(async (request) => request),
      removePackFromFolder: vi.fn(async () => undefined),
      listTags: vi.fn(async () => [{ id: "tag_1", tenantId: "tenant_1", name: "cute", createdAt: "2026-05-09T00:00:00Z" }]),
      createTag: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
      listPackTags: vi.fn(async () => []),
      addTagToPack: vi.fn(async (packId, tagId) => ({ packId, tagId })),
      removeTagFromPack: vi.fn(async () => undefined),
      listSubscriptionGroups: vi.fn(async () => [
        {
          id: "sub_1",
          tenantId: "tenant_1",
          ownerUserId: "user_1",
          title: "Weekly",
          visibility: "private",
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
      createSubscriptionGroup: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
      listSubscriptionGroupPacks: vi.fn(async () => []),
      addPackToSubscriptionGroup: vi.fn(async (request) => request),
      removePackFromSubscriptionGroup: vi.fn(async () => undefined),
      listSubscriptionLinks: vi.fn(async () => [
        {
          id: "packlink",
          tenantId: "tenant_1",
          ownerUserId: "user_1",
          resourceType: "pack",
          resourceId: "pack_1",
          revokedAt: null,
          createdAt: "2026-05-09T00:00:00Z",
          updatedAt: "2026-05-09T00:00:00Z",
        },
      ]),
      createSubscriptionLink: vi.fn(async (request) => ({
        ...request,
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        token: "msm_sub_packlink_secret",
        revokedAt: null,
        createdAt: "2026-05-09T00:00:00Z",
        updatedAt: "2026-05-09T00:00:00Z",
      })),
      rotateSubscriptionLink: vi.fn(async (tokenId) => ({
        id: tokenId,
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        resourceType: "pack",
        resourceId: "pack_1",
        token: "msm_sub_packlink_rotated",
        revokedAt: null,
        createdAt: "2026-05-09T00:00:00Z",
        updatedAt: "2026-05-09T00:00:01Z",
      })),
      revokeSubscriptionLink: vi.fn(async () => undefined),
    };
    const wrapper = mount(ProductMetadataPanel, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        metadataClient: client,
      },
    });

    await flushPromises();
    expect(wrapper.text()).toContain("Favorites");
    expect(wrapper.text()).toContain("cute");
    expect(wrapper.text()).toContain("Weekly");
    expect(wrapper.text()).toContain("packlink");

    await wrapper.get('[aria-label="Folder ID"]').setValue("folder_new");
    await wrapper.get('[aria-label="Folder name"]').setValue("Pinned");
    await wrapper.get('[aria-label="Create folder"]').trigger("click");
    await wrapper.get('[aria-label="Tag ID"]').setValue("tag_new");
    await wrapper.get('[aria-label="Tag name"]').setValue("meme");
    await wrapper.get('[aria-label="Create tag"]').trigger("click");
    await wrapper.get('[aria-label="Subscription group ID"]').setValue("sub_new");
    await wrapper.get('[aria-label="Subscription group title"]').setValue("Public feed");
    await wrapper.get('[aria-label="Subscription group visibility"]').setValue("public");
    await wrapper.get('[aria-label="Create subscription group"]').trigger("click");
    await wrapper.get('[aria-label="Subscription link ID"]').setValue("packlink_new");
    await wrapper.get('[aria-label="Link type"]').setValue("pack");
    await wrapper.get('[aria-label="Resource ID"]').setValue("pack_1");
    await wrapper.get('[aria-label="Create subscription link"]').trigger("click");
    await wrapper.get('[aria-label="Rotate packlink"]').trigger("click");
    await wrapper.get('[aria-label="Revoke packlink"]').trigger("click");
    await flushPromises();

    expect(client.createFolder).toHaveBeenCalledWith({
      id: "folder_new",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      name: "Pinned",
    });
    expect(client.createTag).toHaveBeenCalledWith({ id: "tag_new", tenantId: "tenant_1", name: "meme" });
    expect(client.createSubscriptionGroup).toHaveBeenCalledWith({
      id: "sub_new",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      title: "Public feed",
      visibility: "public",
    });
    expect(client.createSubscriptionLink).toHaveBeenCalledWith({
      id: "packlink_new",
      resourceType: "pack",
      resourceId: "pack_1",
    });
    expect(client.rotateSubscriptionLink).toHaveBeenCalledWith("packlink");
    expect(client.revokeSubscriptionLink).toHaveBeenCalledWith("packlink");
    expect(wrapper.get('[data-testid="subscription-secret"]').text()).toContain("msm_sub_packlink");
    expect(client.listFolders).toHaveBeenCalledTimes(7);
    expect(client.listTags).toHaveBeenCalledTimes(7);
    expect(client.listSubscriptionGroups).toHaveBeenCalledTimes(7);
    expect(client.listSubscriptionLinks).toHaveBeenCalledTimes(7);
  });

  it("adds and removes pack memberships from the Organize workspace", async () => {
    const packs: StickerPackSummary[] = [
      {
        id: "pack_1",
        title: "Sample",
        provider: "Telegram",
        visibility: "private",
        stickerCount: 1,
        subscriptionReady: true,
        updatedAt: "2026-05-09",
      },
    ];
    const client: ProductMetadataClient = {
      listFolders: vi.fn(async () => [
        {
          id: "folder_1",
          tenantId: "tenant_1",
          ownerUserId: "user_1",
          name: "Favorites",
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
      createFolder: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
      listFolderPacks: vi.fn(async () => []),
      addPackToFolder: vi.fn(async (request) => request),
      removePackFromFolder: vi.fn(async () => undefined),
      listTags: vi.fn(async () => [{ id: "tag_1", tenantId: "tenant_1", name: "cute", createdAt: "2026-05-09T00:00:00Z" }]),
      createTag: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
      listPackTags: vi.fn(async () => []),
      addTagToPack: vi.fn(async (packId, tagId) => ({ packId, tagId })),
      removeTagFromPack: vi.fn(async () => undefined),
      listSubscriptionGroups: vi.fn(async () => [
        {
          id: "sub_1",
          tenantId: "tenant_1",
          ownerUserId: "user_1",
          title: "Weekly",
          visibility: "private",
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
      createSubscriptionGroup: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
      listSubscriptionGroupPacks: vi.fn(async () => []),
      addPackToSubscriptionGroup: vi.fn(async (request) => request),
      removePackFromSubscriptionGroup: vi.fn(async () => undefined),
      listSubscriptionLinks: vi.fn(async () => []),
      createSubscriptionLink: vi.fn(),
      rotateSubscriptionLink: vi.fn(),
      revokeSubscriptionLink: vi.fn(),
    };
    const wrapper = mount(ProductMetadataPanel, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        metadataClient: client,
        packs,
      },
    });

    await flushPromises();
    await wrapper.get('[aria-label="Pack to organize"]').setValue("pack_1");
    await wrapper.get('[aria-label="Folder membership target"]').setValue("folder_1");
    await wrapper.get('[aria-label="Add pack to folder"]').trigger("click");
    await wrapper.get('[aria-label="Remove pack from folder"]').trigger("click");
    await wrapper.get('[aria-label="Tag membership target"]').setValue("tag_1");
    await wrapper.get('[aria-label="Add tag to pack"]').trigger("click");
    await wrapper.get('[aria-label="Remove tag from pack"]').trigger("click");
    await wrapper.get('[aria-label="Subscription membership target"]').setValue("sub_1");
    await wrapper.get('[aria-label="Add pack to subscription group"]').trigger("click");
    await wrapper.get('[aria-label="Remove pack from subscription group"]').trigger("click");
    await flushPromises();

    expect(client.addPackToFolder).toHaveBeenCalledWith({ folderId: "folder_1", packId: "pack_1", sortOrder: 0 });
    expect(client.removePackFromFolder).toHaveBeenCalledWith("folder_1", "pack_1");
    expect(client.addTagToPack).toHaveBeenCalledWith("pack_1", "tag_1");
    expect(client.removeTagFromPack).toHaveBeenCalledWith("pack_1", "tag_1");
    expect(client.addPackToSubscriptionGroup).toHaveBeenCalledWith({
      subscriptionGroupId: "sub_1",
      packId: "pack_1",
      sortOrder: 0,
    });
    expect(client.removePackFromSubscriptionGroup).toHaveBeenCalledWith("sub_1", "pack_1");
  });
});
