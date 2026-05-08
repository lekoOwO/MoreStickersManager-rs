import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import ProductMetadataPanel from "@/components/ProductMetadataPanel.vue";
import type { ProductMetadataClient } from "@/lib/api-client";

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
      listTags: vi.fn(async () => [{ id: "tag_1", tenantId: "tenant_1", name: "cute", createdAt: "2026-05-09T00:00:00Z" }]),
      createTag: vi.fn(async (request) => ({ ...request, createdAt: "2026-05-09T00:00:00Z" })),
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
    expect(client.listFolders).toHaveBeenCalledTimes(4);
    expect(client.listTags).toHaveBeenCalledTimes(4);
    expect(client.listSubscriptionGroups).toHaveBeenCalledTimes(4);
  });
});
