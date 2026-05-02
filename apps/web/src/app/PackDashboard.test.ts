import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import PackDashboard from "./PackDashboard.vue";

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
});
