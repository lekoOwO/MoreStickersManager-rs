import { describe, expect, it } from "vitest";

import { normalizeApiBaseUrl, resolveApiBaseUrl, resolveInitialPatToken } from "./runtime-config";

describe("runtime config", () => {
  it("prefers an explicit API base URL", () => {
    expect(
      resolveApiBaseUrl({
        envBaseUrl: " https://api.example.test/ ",
        isDev: false,
        location: { origin: "https://app.example.test" },
      }),
    ).toBe("https://api.example.test");
  });

  it("keeps local Vite development in mock-preview mode when no API base URL is configured", () => {
    expect(
      resolveApiBaseUrl({
        envBaseUrl: "",
        isDev: true,
        location: { origin: "http://127.0.0.1:5173" },
      }),
    ).toBe("");
  });

  it("uses the browser origin for production embedded deployments without a build-time API URL", () => {
    expect(
      resolveApiBaseUrl({
        envBaseUrl: "",
        isDev: false,
        location: { origin: "https://msm.example.test" },
      }),
    ).toBe("https://msm.example.test");
  });

  it("ignores opaque file origins", () => {
    expect(
      resolveApiBaseUrl({
        envBaseUrl: "",
        isDev: false,
        location: { origin: "null" },
      }),
    ).toBe("");
  });

  it("normalizes trailing slash variants", () => {
    expect(normalizeApiBaseUrl("https://msm.example.test///")).toBe("https://msm.example.test");
  });

  it("prefers a stored PAT over a development seed token", () => {
    expect(
      resolveInitialPatToken({
        envPat: "msm_pat_dev_seed",
        isDev: true,
        storage: { getItem: () => "msm_pat_browser" },
      }),
    ).toBe("msm_pat_browser");
  });

  it("uses VITE_MSM_PAT only for local development seeding", () => {
    expect(
      resolveInitialPatToken({
        envPat: "msm_pat_dev_seed",
        isDev: true,
        storage: { getItem: () => null },
      }),
    ).toBe("msm_pat_dev_seed");
  });

  it("does not embed a build-time PAT in production", () => {
    expect(
      resolveInitialPatToken({
        envPat: "msm_pat_should_not_ship",
        isDev: false,
        storage: { getItem: () => null },
      }),
    ).toBe("");
  });
});
