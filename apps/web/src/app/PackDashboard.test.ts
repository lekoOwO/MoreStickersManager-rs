import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import PackDashboard from "./PackDashboard.vue";
import type { PackClient } from "@/lib/api-client";

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
});
