import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import TenantAdminPanel from "@/components/TenantAdminPanel.vue";
import type { TenantAdminClient } from "@/lib/api-client";

describe("tenant admin UI", () => {
  it("lists tenant members and updates member roles", async () => {
    const client: TenantAdminClient = {
      listTenantMembers: vi.fn(async () => [
        {
          tenantId: "tenant_1",
          userId: "user_1",
          role: "admin",
          createdAt: "2026-05-09T00:00:00Z",
        },
        {
          tenantId: "tenant_1",
          userId: "user_2",
          role: "user",
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
      setTenantMemberRole: vi.fn(async (tenantId, userId, role) => ({
        tenantId,
        userId,
        role,
        createdAt: "2026-05-09T00:00:00Z",
      })),
    };
    const wrapper = mount(TenantAdminPanel, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        patToken: "msm_pat_admin_secret",
        tenantAdminClient: client,
      },
    });

    await flushPromises();
    expect(wrapper.text()).toContain("Tenant admin");
    expect(wrapper.text()).toContain("user_1");
    expect(wrapper.text()).toContain("user_2");

    await wrapper.get('[aria-label="Member user ID"]').setValue("user_3");
    await wrapper.get('[aria-label="Member role"]').setValue("admin");
    await wrapper.get('[aria-label="Set member role"]').trigger("click");
    await flushPromises();

    expect(client.setTenantMemberRole).toHaveBeenCalledWith("tenant_1", "user_3", "admin");
    expect(client.listTenantMembers).toHaveBeenCalledTimes(2);
  });
});
