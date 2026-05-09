import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import PackDashboard from "./PackDashboard.vue";
import type { PackClient, TenantAdminClient } from "@/lib/api-client";

describe("PackDashboard", () => {
  it("renders mock pack metrics and provider labels", async () => {
    const wrapper = mount(PackDashboard, {
      props: {
        locale: "en",
      },
    });

    await flushPromises();

    expect(wrapper.text()).toContain("Packs");
    expect(wrapper.text()).toContain("120");
    expect(wrapper.text()).toContain("Telegram");
    expect(wrapper.text()).toContain("LINE Stickers");
    expect(wrapper.text()).toContain("LINE Emojis");
    expect(wrapper.text()).toContain("Public");
    expect(wrapper.text()).toContain("Private");
    expect(wrapper.text()).toContain("Members");
  });

  it("renames and deletes packs through the injected client", async () => {
    const client: PackClient = {
      listStickerPacks: vi.fn(async () => [
        {
          id: "pack_1",
          title: "API Cats",
          provider: "Telegram",
          visibility: "private",
          stickerCount: 2,
          subscriptionReady: true,
          updatedAt: "2026-05-04",
        },
      ]),
      importStickerPack: vi.fn(async () => {}),
      updateStickerPack: vi.fn(async () => {}),
      deleteStickerPack: vi.fn(async () => {}),
    };
    const wrapper = mount(PackDashboard, {
      props: {
        locale: "en",
        packClient: client,
      },
    });

    await flushPromises();
    const titleInput = wrapper.get('[aria-label="Pack title"]');
    await titleInput.setValue("Renamed Pack");
    await wrapper.get('[aria-label="Pack visibility"]').setValue("public");
    await wrapper.get('[aria-label="Save pack changes"]').trigger("click");
    await flushPromises();
    await wrapper.get('[aria-label="Delete pack"]').trigger("click");
    await flushPromises();

    expect(client.updateStickerPack).toHaveBeenCalledWith({
      packId: "pack_1",
      title: "Renamed Pack",
      visibility: "public",
    });
    expect(client.deleteStickerPack).toHaveBeenCalledWith("pack_1");
    expect(client.listStickerPacks).toHaveBeenCalledTimes(3);
  });

  it("imports sticker packs through the injected client", async () => {
    const client: PackClient = {
      listStickerPacks: vi.fn(async () => []),
      importStickerPack: vi.fn(async () => {}),
      updateStickerPack: vi.fn(async () => {}),
      deleteStickerPack: vi.fn(async () => {}),
    };
    const pack = {
      id: "MoreStickers:Telegram:Pack:cats",
      title: "Cats",
      logo: {
        id: "sticker_1",
        title: "cat",
        image: "https://msm.example/cat.webp",
        sticker_pack_id: "MoreStickers:Telegram:Pack:cats",
      },
      stickers: [],
    };
    const wrapper = mount(PackDashboard, {
      props: {
        locale: "en",
        packClient: client,
        tenantId: "tenant_1",
        ownerUserId: "user_1",
      },
    });

    await flushPromises();
    await wrapper.get('[aria-label="Open import sticker pack dialog"]').trigger("click");
    await wrapper.get('[aria-label="Import pack ID"]').setValue("pack_1");
    await wrapper.get('[aria-label="Import pack JSON"]').setValue(JSON.stringify(pack));
    await wrapper.get('button[aria-label="Import sticker pack"]').trigger("click");
    await flushPromises();

    expect(client.importStickerPack).toHaveBeenCalledWith({
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      packId: "pack_1",
      visibility: "private",
      pack,
    });
    expect(client.listStickerPacks).toHaveBeenCalledTimes(2);
  });

  it("renders tenant administration workspace when selected by the shell", async () => {
    const packClient: PackClient = {
      listStickerPacks: vi.fn(async () => []),
      importStickerPack: vi.fn(async () => {}),
      updateStickerPack: vi.fn(async () => {}),
      deleteStickerPack: vi.fn(async () => {}),
    };
    const tenantAdminClient: TenantAdminClient = {
      listTenantMembers: vi.fn(async () => [
        {
          tenantId: "tenant_1",
          userId: "user_1",
          role: "admin",
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
        localRegistrationEnabled: true,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      updateTenantSettings: vi.fn(async (tenantId, request) => ({
        tenantId,
        name: request.name,
        publicAssetUrl: request.publicAssetUrl,
        localRegistrationEnabled: request.localRegistrationEnabled,
        createdAt: "2026-05-09T00:00:00Z",
      })),
      setTenantUserStatus: vi.fn(async (tenantId, userId, isDisabled) => ({
        id: userId,
        email: "member@example.com",
        displayName: tenantId,
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
    const wrapper = mount(PackDashboard, {
      props: {
        locale: "en",
        activeSection: "admin",
        packClient,
        tenantAdminClient,
        tenantId: "tenant_1",
      },
    });

    await flushPromises();

    expect(wrapper.text()).toContain("Tenant admin");
    expect(wrapper.text()).toContain("user_1");
    expect(tenantAdminClient.listTenantMembers).toHaveBeenCalledWith("tenant_1");
  });
});
