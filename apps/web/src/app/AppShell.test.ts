import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";

import AppShell from "./AppShell.vue";

describe("AppShell PAT scope policy", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
    vi.unstubAllGlobals();
    window.localStorage.clear();
    window.history.pushState({}, "", "/");
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

  it("completes OIDC login from the auth dialog and stores the returned PAT", async () => {
    vi.stubEnv("VITE_MSM_API_BASE_URL", "https://msm.example.test");
    const fetchImpl = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
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
      if (url.endsWith("/api/v1/auth/oidc/callback") && init?.method === "POST") {
        return new Response(
          JSON.stringify({
            id: "web_oidc",
            userId: "oidc-user",
            name: "Web OIDC",
            token: "msm_pat_oidc_secret",
            scopes: ["pack.read"],
            expiresAt: null,
            revokedAt: null,
            createdAt: "2026-05-10T00:00:00Z",
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
    await wrapper.get('[aria-label="Start OIDC login"]').trigger("click");
    await flushPromises();

    await wrapper.get('[aria-label="OIDC authorization code"]').setValue("provider-code-1");
    await wrapper.get('[aria-label="OIDC issuer"]').setValue("https://accounts.google.com");
    await wrapper.get('[aria-label="OIDC audience"]').setValue("client_1");
    await wrapper.get('[aria-label="OIDC provider subject"]').setValue("subject_1");
    await wrapper.get('[aria-label="OIDC email"]').setValue("user@example.test");
    await wrapper.get('[aria-label="OIDC display name"]').setValue("OIDC User");
    await wrapper.get('[aria-label="Complete OIDC login"]').trigger("click");
    await flushPromises();

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/auth/oidc/callback", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        state: "state_1",
        nonce: "nonce_1",
        authorizationCode: "provider-code-1",
        issuer: "https://accounts.google.com",
        audience: "client_1",
        providerSubject: "subject_1",
        email: "user@example.test",
        displayName: "OIDC User",
        tokenId: "webui-oidc",
        tokenName: "Web OIDC",
        scopes: ["pack.read"],
        expiresAt: null,
      }),
    });
    expect(wrapper.emitted("updatePatToken")?.at(-1)).toEqual(["msm_pat_oidc_secret"]);
    expect(wrapper.text()).toContain("msm_pat_oidc_secret");
  });

  it("prefills OIDC callback fields from the redirect URL and pending login state", async () => {
    vi.stubEnv("VITE_MSM_API_BASE_URL", "https://msm.example.test");
    window.localStorage.setItem(
      "msm.oidc.pending",
      JSON.stringify({
        state: "state_2",
        nonce: "nonce_2",
        tenantId: "tenant_1",
        providerId: "google",
        redirectUri: "https://app.example.test/auth/oidc/callback",
        expiresAt: "2026-05-10T00:10:00Z",
      }),
    );
    window.history.pushState({}, "", "/auth/oidc/callback?code=provider-code-2&state=state_2");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => new Response(JSON.stringify({ userId: "user_1", allowedScopes: ["pack.read"] }), { status: 200 })),
    );

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

    expect((wrapper.get('[aria-label="OIDC authorization code"]').element as HTMLInputElement).value).toBe(
      "provider-code-2",
    );
    expect((wrapper.get('[aria-label="OIDC state"]').element as HTMLInputElement).value).toBe("state_2");
    expect((wrapper.get('[aria-label="OIDC nonce"]').element as HTMLInputElement).value).toBe("nonce_2");
    expect((wrapper.get('[aria-label="OIDC tenant ID"]').element as HTMLInputElement).value).toBe("tenant_1");
    expect((wrapper.get('[aria-label="OIDC provider ID"]').element as HTMLInputElement).value).toBe("google");
  });
});
