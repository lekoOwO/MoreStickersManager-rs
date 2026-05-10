import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import ProviderImportPlanner from "@/components/ProviderImportPlanner.vue";
import type { ProviderImportClient } from "@/lib/api-client";

describe("provider import UI", () => {
  it("creates provider import plans and displays the fetch boundary", async () => {
    const client: ProviderImportClient = {
      createProviderImportPlan: vi.fn(async (request) => ({
        providerId: request.providerId,
        remoteId: request.remoteId,
        metadataRequest: {
          method: "GET",
          url: "https://store.line.me/stickershop/product/12345/en",
          redactedHeaders: [],
        },
        assetStrategy: "directRemoteUrls",
      })),
    };
    const wrapper = mount(ProviderImportPlanner, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        providerImportClient: client,
      },
    });

    await wrapper.get('[aria-label="Provider"]').setValue("line-stickers");
    await wrapper.get('[aria-label="Remote pack ID"]').setValue("12345");
    await wrapper.get('[aria-label="Provider base URL"]').setValue("https://store.line.me");
    await wrapper.get("form").trigger("submit");
    await flushPromises();

    expect(client.createProviderImportPlan).toHaveBeenCalledWith({
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      providerId: "line-stickers",
      remoteId: "12345",
      baseUrl: "https://store.line.me",
    });
    expect(wrapper.text()).toContain("Direct remote image URLs");
    expect(wrapper.text()).toContain("https://store.line.me/stickershop/product/12345/en");
  });
});
