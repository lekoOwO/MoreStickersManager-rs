import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import ExportJobTimeline from "@/components/ExportJobTimeline.vue";
import ExportTargetPanel from "@/components/ExportTargetPanel.vue";
import PackExportWizard from "@/components/PackExportWizard.vue";
import type { ExportClient, ExportJob, ExportTarget } from "@/lib/exportApi";
import type { StickerPackSummary } from "@/lib/sticker-packs";

describe("export UI", () => {
  it("creates export targets and displays redacted config", async () => {
    const target = sampleTarget();
    const client: ExportClient = {
      listExportTargetKinds: vi.fn(async () => [
        {
          kind: "telegram",
          displayName: "Telegram",
          supportsRemotePublication: true,
          supportsMediaConversion: true,
          requiresCredentials: true,
        },
      ]),
      listExportTargets: vi.fn(async () => [target]),
      createExportTarget: vi.fn(async () => target),
      updateExportTarget: vi.fn(),
      deleteExportTarget: vi.fn(),
      createExportJob: vi.fn(),
      getExportJob: vi.fn(),
      listExportJobEvents: vi.fn(),
    };
    const wrapper = mount(ExportTargetPanel, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        exportClient: client,
      },
    });

    await flushPromises();
    await wrapper.get('[aria-label="Target ID"]').setValue("target_telegram");
    await wrapper.get('[aria-label="Target name"]').setValue("Telegram");
    await wrapper.get('[aria-label="Target config JSON"]').setValue('{"botToken":"123:secret"}');
    await wrapper.get('[aria-label="Create export target"]').trigger("click");
    await flushPromises();

    expect(client.createExportTarget).toHaveBeenCalledWith({
      id: "target_telegram",
      tenantId: "tenant_1",
      kind: "telegram",
      name: "Telegram",
      config: { botToken: "123:secret" },
      isEnabled: true,
    });
    expect(wrapper.text()).toContain("<redacted>");
  });

  it("validates Telegram bot tokens before creating export targets", async () => {
    const client: ExportClient = {
      listExportTargetKinds: vi.fn(async () => [
        {
          kind: "telegram",
          displayName: "Telegram",
          supportsRemotePublication: true,
          supportsMediaConversion: true,
          requiresCredentials: true,
        },
      ]),
      listExportTargets: vi.fn(async () => []),
      createExportTarget: vi.fn(),
      updateExportTarget: vi.fn(),
      deleteExportTarget: vi.fn(),
      createExportJob: vi.fn(),
      getExportJob: vi.fn(),
      listExportJobEvents: vi.fn(),
    };
    const wrapper = mount(ExportTargetPanel, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        exportClient: client,
      },
    });

    await flushPromises();
    await wrapper.get('[aria-label="Target config JSON"]').setValue('{"botToken":"bad"}');
    await wrapper.get('[aria-label="Create export target"]').trigger("click");
    await flushPromises();

    expect(client.createExportTarget).not.toHaveBeenCalled();
    expect(wrapper.text()).toContain("Telegram bot token must match");
  });

  it("queues export jobs and renders job events", async () => {
    const job = sampleJob();
    const packs: StickerPackSummary[] = [
      {
        id: "pack_1",
        title: "Cats",
        provider: "Telegram",
        visibility: "private",
        stickerCount: 2,
        subscriptionReady: true,
        updatedAt: "2026-05-07",
      },
    ];
    const client: ExportClient = {
      listExportTargetKinds: vi.fn(),
      listExportTargets: vi.fn(async () => [sampleTarget()]),
      createExportTarget: vi.fn(),
      updateExportTarget: vi.fn(),
      deleteExportTarget: vi.fn(),
      createExportJob: vi.fn(async () => job),
      getExportJob: vi.fn(async () => job),
      listExportJobEvents: vi.fn(async () => [
        {
          jobId: "job_1",
          sequence: 1,
          level: "info",
          stage: "queued",
          message: "job queued",
          metadata: {},
          createdAt: "2026-05-07T00:00:00Z",
        },
      ]),
    };
    const wrapper = mount(PackExportWizard, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        packs,
        exportClient: client,
      },
    });

    await flushPromises();
    await wrapper.get('[aria-label="Export job ID"]').setValue("job_1");
    await wrapper.get('[aria-label="Queue export job"]').trigger("click");
    await flushPromises();

    expect(client.createExportJob).toHaveBeenCalledWith({
      id: "job_1",
      tenantId: "tenant_1",
      sourcePackId: "pack_1",
      targetId: "target_telegram",
      options: {},
    });
    expect(wrapper.text()).toContain("job queued");
  });

  it("renders export job conflict errors", async () => {
    const packs: StickerPackSummary[] = [
      {
        id: "pack_1",
        title: "Cats",
        provider: "Telegram",
        visibility: "private",
        stickerCount: 2,
        subscriptionReady: true,
        updatedAt: "2026-05-07",
      },
    ];
    const client: ExportClient = {
      listExportTargetKinds: vi.fn(),
      listExportTargets: vi.fn(async () => [sampleTarget()]),
      createExportTarget: vi.fn(),
      updateExportTarget: vi.fn(),
      deleteExportTarget: vi.fn(),
      createExportJob: vi.fn(async () => {
        throw new Error("Telegram set already exists");
      }),
      getExportJob: vi.fn(),
      listExportJobEvents: vi.fn(),
    };
    const wrapper = mount(PackExportWizard, {
      props: {
        locale: "en",
        tenantId: "tenant_1",
        packs,
        exportClient: client,
      },
    });

    await flushPromises();
    await wrapper.get('[aria-label="Queue export job"]').trigger("click");
    await flushPromises();

    expect(wrapper.text()).toContain("Telegram set already exists");
  });

  it("renders export job timelines", () => {
    const wrapper = mount(ExportJobTimeline, {
      props: {
        locale: "en",
        job: sampleJob(),
        events: [
          {
            jobId: "job_1",
            sequence: 1,
            level: "info",
            stage: "queued",
            message: "job queued",
            metadata: {},
            createdAt: "2026-05-07T00:00:00Z",
          },
        ],
      },
    });

    expect(wrapper.text()).toContain("queued");
    expect(wrapper.text()).toContain("job queued");
  });
});

function sampleTarget(): ExportTarget {
  return {
    id: "target_telegram",
    tenantId: "tenant_1",
    kind: "telegram",
    name: "Telegram",
    config: { botToken: "<redacted>" },
    isEnabled: true,
    createdAt: "2026-05-07T00:00:00Z",
    updatedAt: "2026-05-07T00:00:00Z",
  };
}

function sampleJob(): ExportJob {
  return {
    id: "job_1",
    tenantId: "tenant_1",
    ownerUserId: "user_1",
    sourcePackId: "pack_1",
    targetId: "target_telegram",
    status: "queued",
    request: { options: {} },
    result: null,
    errorSummary: null,
    createdAt: "2026-05-07T00:00:00Z",
    updatedAt: "2026-05-07T00:00:00Z",
  };
}
