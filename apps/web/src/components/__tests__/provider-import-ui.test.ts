import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import ProviderImportPlanner from "@/components/ProviderImportPlanner.vue";
import type { ProviderImportClient } from "@/lib/api-client";

describe("provider import UI", () => {
  it("creates provider import plans and displays the fetch boundary", async () => {
    const client: ProviderImportClient = {
      createProviderImportPlan: vi.fn(async (request) => ({
        providerId: request.providerId,
        remoteId: request.remoteId,
        metadataRequest: {
          method: "GET",
          url: "https://store.line.me/stickershop/product/12345/en",
          redactedHeaders: [],
        },
        assetStrategy: "directRemoteUrls",
      })),
      createProviderImportJob: vi.fn(async (request) => ({
        id: request.id,
        tenantId: request.tenantId,
        ownerUserId: request.ownerUserId,
        providerId: request.providerId,
        remoteId: request.remoteId,
        targetPackId: request.targetPackId,
        status: "queued",
        request: {},
        result: null,
        errorSummary: null,
        attemptCount: 0,
        maxAttempts: 3,
        nextAttemptAt: null,
        createdAt: "2026-05-10T00:00:00Z",
        updatedAt: "2026-05-10T00:00:00Z",
      })),
      getProviderImportJob: vi.fn(async (jobId) => ({
        id: jobId,
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        providerId: "line-stickers",
        remoteId: "12345",
        targetPackId: "pack_line_12345",
        status: "running",
        request: {},
        result: null,
        errorSummary: null,
        attemptCount: 1,
        maxAttempts: 3,
        nextAttemptAt: null,
        createdAt: "2026-05-10T00:00:00Z",
        updatedAt: "2026-05-10T00:01:00Z",
      })),
      listProviderImportJobEvents: vi.fn(async (jobId) => [
        {
          jobId,
          sequence: 1,
          level: "info",
          stage: "queued",
          message: "Provider import job queued.",
          metadata: {},
          createdAt: "2026-05-10T00:00:00Z",
        },
      ]),
      listProviderConfigs: vi.fn(async () => []),
      upsertProviderConfig: vi.fn(),
      deleteProviderConfig: vi.fn(),
    };
    const wrapper = mount(ProviderImportPlanner, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        providerImportClient: client,
      },
    });

    await wrapper.get('[aria-label="Provider"]').setValue("line-stickers");
    await wrapper.get('[aria-label="Remote pack ID"]').setValue("12345");
    await wrapper.get('[aria-label="Provider base URL"]').setValue("https://store.line.me");
    await wrapper.get("form").trigger("submit");
    await flushPromises();

    expect(client.createProviderImportPlan).toHaveBeenCalledWith({
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      providerId: "line-stickers",
      remoteId: "12345",
      baseUrl: "https://store.line.me",
    });
    expect(wrapper.text()).toContain("Direct remote image URLs");
    expect(wrapper.text()).toContain("https://store.line.me/stickershop/product/12345/en");
  });

  it("queues provider import jobs and displays status events", async () => {
    const client: ProviderImportClient = {
      createProviderImportPlan: vi.fn(),
      createProviderImportJob: vi.fn(async (request) => ({
        id: request.id,
        tenantId: request.tenantId,
        ownerUserId: request.ownerUserId,
        providerId: request.providerId,
        remoteId: request.remoteId,
        targetPackId: request.targetPackId,
        status: "queued",
        request: {},
        result: null,
        errorSummary: null,
        attemptCount: 0,
        maxAttempts: 3,
        nextAttemptAt: null,
        createdAt: "2026-05-10T00:00:00Z",
        updatedAt: "2026-05-10T00:00:00Z",
      })),
      getProviderImportJob: vi.fn(async (jobId) => ({
        id: jobId,
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        providerId: "line-stickers",
        remoteId: "12345",
        targetPackId: "pack_line_12345",
        status: "succeeded",
        request: {},
        result: { packId: "pack_line_12345" },
        errorSummary: null,
        attemptCount: 1,
        maxAttempts: 3,
        nextAttemptAt: null,
        createdAt: "2026-05-10T00:00:00Z",
        updatedAt: "2026-05-10T00:01:00Z",
      })),
      listProviderImportJobEvents: vi.fn(async (jobId) => [
        {
          jobId,
          sequence: 1,
          level: "info",
          stage: "queued",
          message: "Provider import job queued.",
          metadata: {},
          createdAt: "2026-05-10T00:00:00Z",
        },
        {
          jobId,
          sequence: 2,
          level: "info",
          stage: "succeeded",
          message: "Provider import job completed.",
          metadata: {},
          createdAt: "2026-05-10T00:01:00Z",
        },
      ]),
      listProviderConfigs: vi.fn(async () => []),
      upsertProviderConfig: vi.fn(),
      deleteProviderConfig: vi.fn(),
    };
    const wrapper = mount(ProviderImportPlanner, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        providerImportClient: client,
      },
    });

    await wrapper.get('[aria-label="Provider"]').setValue("line-stickers");
    await wrapper.get('[aria-label="Remote pack ID"]').setValue("12345");
    await wrapper.get('[aria-label="Provider base URL"]').setValue("https://store.line.me");
    await wrapper.get('[aria-label="Import job ID"]').setValue("provider_job_1");
    await wrapper.get('[aria-label="Target pack ID"]').setValue("pack_line_12345");
    await wrapper.get('[aria-label="Queue import job"]').trigger("click");
    await flushPromises();

    expect(client.createProviderImportJob).toHaveBeenCalledWith({
      id: "provider_job_1",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      providerId: "line-stickers",
      remoteId: "12345",
      targetPackId: "pack_line_12345",
      baseUrl: "https://store.line.me",
    });
    expect(client.getProviderImportJob).toHaveBeenCalledWith("provider_job_1");
    expect(client.listProviderImportJobEvents).toHaveBeenCalledWith("provider_job_1");
    expect(wrapper.get('[data-testid="provider-import-job-panel"]').text()).toContain("succeeded");
    expect(wrapper.text()).toContain("Provider import job completed.");
  });

  it("manages provider configs and displays redacted secrets", async () => {
    const client: ProviderImportClient = {
      createProviderImportPlan: vi.fn(),
      createProviderImportJob: vi.fn(),
      getProviderImportJob: vi.fn(),
      listProviderImportJobEvents: vi.fn(),
      listProviderConfigs: vi.fn(async () => [
        {
          id: "provider_telegram",
          tenantId: "tenant_1",
          providerId: "telegram",
          name: "Telegram Import Bot",
          config: { botToken: "<redacted>" },
          isEnabled: true,
          createdAt: "2026-05-10T00:00:00Z",
          updatedAt: "2026-05-10T00:00:00Z",
        },
      ]),
      upsertProviderConfig: vi.fn(async (id, request) => ({
        id,
        tenantId: request.tenantId,
        providerId: request.providerId,
        name: request.name,
        config: { botToken: "<redacted>", nested: { clientSecret: "<redacted>" } },
        isEnabled: request.isEnabled,
        createdAt: "2026-05-10T00:00:00Z",
        updatedAt: "2026-05-10T00:01:00Z",
      })),
      deleteProviderConfig: vi.fn(async () => undefined),
    };
    const wrapper = mount(ProviderImportPlanner, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        ownerUserId: "user_1",
        providerImportClient: client,
      },
    });
    await flushPromises();

    expect(client.listProviderConfigs).toHaveBeenCalledWith("tenant_1");
    expect(wrapper.text()).toContain("Telegram Import Bot");
    expect(wrapper.text()).toContain("<redacted>");

    await wrapper.get('[aria-label="Provider config ID"]').setValue("provider_line");
    await wrapper.get('[aria-label="Provider config source"]').setValue("line-stickers");
    await wrapper.get('[aria-label="Provider config name"]').setValue("LINE Store");
    await wrapper.get('[aria-label="Provider config JSON"]').setValue('{"botToken":"123456:secret","nested":{"clientSecret":"secret_1"}}');
    await wrapper.get('[aria-label="Save provider config"]').trigger("click");
    await flushPromises();

    expect(client.upsertProviderConfig).toHaveBeenCalledWith("provider_line", {
      tenantId: "tenant_1",
      providerId: "line-stickers",
      name: "LINE Store",
      config: { botToken: "123456:secret", nested: { clientSecret: "secret_1" } },
      isEnabled: true,
    });
    expect(wrapper.text()).toContain("LINE Store");
    expect(wrapper.text()).toContain("<redacted>");

    await wrapper.get('[aria-label="Delete provider config provider_line"]').trigger("click");
    await flushPromises();

    expect(client.deleteProviderConfig).toHaveBeenCalledWith("provider_line");
  });
});
