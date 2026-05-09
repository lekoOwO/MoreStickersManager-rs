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
      getTenantSettings: vi.fn(async () => ({
        tenantId: "tenant_1",
        name: "Default tenant",
        publicAssetUrl: null,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      updateTenantSettings: vi.fn(async (tenantId, request) => ({
        tenantId,
        name: request.name,
        publicAssetUrl: request.publicAssetUrl,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      setTenantUserStatus: vi.fn(async (tenantId, userId, isDisabled) => ({
        id: userId,
        email: "member@example.com",
        displayName: "Member",
        isDisabled,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      listTenantRoles: vi.fn(async () => []),
      upsertTenantRole: vi.fn(async (tenantId, roleId, request) => ({
        id: roleId,
        tenantId,
        name: request.name,
        permissions: request.permissions,
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

  it("manages tenant settings, user status, and role templates", async () => {
    const client: TenantAdminClient = {
      listTenantMembers: vi.fn(async () => [
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
      getTenantSettings: vi.fn(async () => ({
        tenantId: "tenant_1",
        name: "Default tenant",
        publicAssetUrl: null,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      updateTenantSettings: vi.fn(async (tenantId, request) => ({
        tenantId,
        name: request.name,
        publicAssetUrl: request.publicAssetUrl,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      setTenantUserStatus: vi.fn(async (tenantId, userId, isDisabled) => ({
        id: userId,
        email: "member@example.com",
        displayName: "Member",
        isDisabled,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      listTenantRoles: vi.fn(async () => [
        {
          id: "role_viewer",
          tenantId: "tenant_1",
          name: "Viewers",
          permissions: ["pack.read"],
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
      upsertTenantRole: vi.fn(async (tenantId, roleId, request) => ({
        id: roleId,
        tenantId,
        name: request.name,
        permissions: request.permissions,
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
    await wrapper.get('[aria-label="Tenant name"]').setValue("Production");
    await wrapper.get('[aria-label="Public asset URL"]').setValue("https://cdn.example.test/msm");
    await wrapper.get('[aria-label="Save tenant settings"]').trigger("click");
    await wrapper.get('[aria-label="User status ID"]').setValue("user_2");
    await wrapper.get('[aria-label="Disable user"]').trigger("click");
    await wrapper.get('[aria-label="Role template ID"]').setValue("role_editor");
    await wrapper.get('[aria-label="Role template name"]').setValue("Editors");
    await wrapper.get('[aria-label="Permission: Update packs"]').setValue(true);
    await wrapper.get('[aria-label="Save role template"]').trigger("click");
    await flushPromises();

    expect(client.updateTenantSettings).toHaveBeenCalledWith("tenant_1", {
      name: "Production",
      publicAssetUrl: "https://cdn.example.test/msm",
    });
    expect(client.setTenantUserStatus).toHaveBeenCalledWith("tenant_1", "user_2", true);
    expect(client.upsertTenantRole).toHaveBeenCalledWith("tenant_1", "role_editor", {
      name: "Editors",
      permissions: ["pack.update"],
    });
    expect(wrapper.text()).toContain("Viewers");
  });
});
