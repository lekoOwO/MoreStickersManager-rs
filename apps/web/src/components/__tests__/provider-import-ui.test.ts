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
});
