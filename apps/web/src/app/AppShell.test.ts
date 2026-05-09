import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";

import AppShell from "./AppShell.vue";

describe("AppShell PAT scope policy", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.unstubAllGlobals();
  });

  it("loads role-allowed PAT scopes before rendering the PAT picker", async () => {
    vi.stubEnv("VITE_MSM_API_BASE_URL", "https://msm.example.test");
    vi.stubEnv("VITE_MSM_USER_ID", "user_1");
    const fetchImpl = vi.fn(async (input: RequestInfo | URL) => {
      const url = input.toString();
      if (url.includes("/api/v1/pats/scope-policy")) {
        return new Response(
          JSON.stringify({
            userId: "user_1",
            allowedScopes: ["pack.read", "pat.manage"],
          }),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify([]), { status: 200 });
    });
    vi.stubGlobal("fetch", fetchImpl);

    const wrapper = mount(AppShell, {
      props: {
        locale: "en",
        patToken: "msm_pat_admin_secret",
        theme: "light",
      },
      global: {
        stubs: {
          PackDashboard: { template: "<main />" },
        },
      },
    });

    await flushPromises();
    const patButton = wrapper.findAll("button").find((button) => button.text().trim() === "PAT");
    expect(patButton).toBeTruthy();
    await patButton!.trigger("click");
    await flushPromises();

    expect(fetchImpl).toHaveBeenCalledWith(
      "https://msm.example.test/api/v1/pats/scope-policy?userId=user_1",
      {
        headers: {
          Authorization: "Bearer msm_pat_admin_secret",
        },
      },
    );
    expect(wrapper.text()).toContain("Read packs");
    expect(wrapper.text()).toContain("Manage PATs");
    expect(wrapper.text()).not.toContain("Manage tenant users");
  });
});
