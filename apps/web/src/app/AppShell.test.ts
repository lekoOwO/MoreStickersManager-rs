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

  it("starts OIDC login from the auth dialog", async () => {
    vi.stubEnv("VITE_MSM_API_BASE_URL", "https://msm.example.test");
    const fetchImpl = vi.fn(async (input: RequestInfo | URL) => {
      const url = input.toString();
      if (url.includes("/api/v1/auth/oidc/tenant_1/google/login")) {
        return new Response(
          JSON.stringify({
            tenantId: "tenant_1",
            providerId: "google",
            authorizationUrl: "https://accounts.google.com/o/oauth2/v2/auth?state=state_1",
            state: "state_1",
            nonce: "nonce_1",
            expiresAt: "2026-05-10T00:10:00Z",
          }),
          { status: 200 },
        );
      }
      return new Response(JSON.stringify({ userId: "user_1", allowedScopes: ["pack.read"] }), { status: 200 });
    });
    vi.stubGlobal("fetch", fetchImpl);

    const wrapper = mount(AppShell, {
      props: {
        locale: "en",
        patToken: "",
        theme: "light",
      },
      global: {
        stubs: {
          PackDashboard: { template: "<main />" },
        },
      },
    });

    await flushPromises();
    const loginButton = wrapper.findAll("button").find((button) => button.text().trim() === "Local Login");
    expect(loginButton).toBeTruthy();
    await loginButton!.trigger("click");
    await wrapper.get('[aria-label="OIDC tenant ID"]').setValue("tenant_1");
    await wrapper.get('[aria-label="OIDC provider ID"]').setValue("google");
    await wrapper.get('[aria-label="OIDC redirect URI"]').setValue("https://app.example.test/callback");
    await wrapper.get('[aria-label="Start OIDC login"]').trigger("click");
    await flushPromises();

    expect(fetchImpl).toHaveBeenCalledWith(
      "https://msm.example.test/api/v1/auth/oidc/tenant_1/google/login?redirectUri=https%3A%2F%2Fapp.example.test%2Fcallback",
    );
    expect(wrapper.text()).toContain("Open provider authorization URL");
    expect(wrapper.text()).toContain("state_1");
  });
});
