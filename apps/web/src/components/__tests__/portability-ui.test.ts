import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import PortabilityPanel from "@/components/PortabilityPanel.vue";
import type { PortabilityClient } from "@/lib/api-client";

describe("PortabilityPanel", () => {
  it("exports and imports portable user data", async () => {
    const exported = {
      version: 1,
      user: { id: "user_1", email: "leko@example.com", displayName: "Leko" },
      packs: [],
      subscriptionGroups: [],
    };
    const client: PortabilityClient = {
      exportUserData: vi.fn(async () => exported),
      importUserData: vi.fn(async () => undefined),
    };
    const wrapper = mount(PortabilityPanel, {
      props: {
        locale: "en",
        patToken: "msm_pat_web_secret",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        portabilityClient: client,
      },
    });

    await wrapper.get("button").trigger("click");
    expect(client.exportUserData).toHaveBeenCalledWith("user_1");
    expect(wrapper.get("textarea").element.value).toContain('"version": 1');

    await wrapper.get('[aria-label="Target tenant ID"]').setValue("tenant_2");
    await wrapper.findAll("button")[1].trigger("click");

    expect(client.importUserData).toHaveBeenCalledWith({
      tenantId: "tenant_2",
      export: exported,
    });
    expect(wrapper.text()).toContain("Portable user data imported.");
  });
});
