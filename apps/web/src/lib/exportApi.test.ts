import { describe, expect, it, vi } from "vitest";

import {
  createExportClient,
  exportJobResultLink,
  exportTargetListUrl,
  telegramPublicationListUrl,
  type ExportTarget,
} from "./exportApi";

describe("export API client", () => {
  it("constructs target list URLs with encoded tenant IDs", () => {
    expect(exportTargetListUrl("https://msm.example.test/", "tenant 1")).toBe(
      "https://msm.example.test/api/v1/export-targets?tenantId=tenant+1",
    );
  });

  it("constructs Telegram publication list URLs with encoded pack IDs", () => {
    expect(telegramPublicationListUrl("https://msm.example.test/", "pack 1")).toBe(
      "https://msm.example.test/api/v1/telegram-publications?packId=pack+1",
    );
  });

  it("extracts Telegram publication result links", () => {
    expect(
      exportJobResultLink({
        kind: "telegramPublished",
        stickerSetUrl: "https://t.me/addstickers/sample",
      }),
    ).toBe("https://t.me/addstickers/sample");
    expect(
      exportJobResultLink({
        kind: "telegramPublished",
        url: "https://t.me/addstickers/fallback",
      }),
    ).toBe("https://t.me/addstickers/fallback");
  });

  it("lists export target kinds and sends bearer auth", async () => {
    const fetchImpl = vi.fn(async () => {
      return new Response(
        JSON.stringify([
          {
            kind: "telegram",
            displayName: "Telegram",
            supportsRemotePublication: true,
            supportsMediaConversion: true,
            requiresCredentials: true,
          },
        ]),
        { status: 200 },
      );
    });
    const client = createExportClient({
      baseUrl: "https://msm.example.test",
      authToken: "msm_pat_web_secret",
      fetchImpl,
    });

    const kinds = await client.listExportTargetKinds();

    expect(kinds[0]?.kind).toBe("telegram");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/export-target-kinds", {
      headers: {
        Authorization: "Bearer msm_pat_web_secret",
      },
    });
  });

  it("creates export jobs and reads job events", async () => {
    const job = {
      id: "job_1",
      tenantId: "tenant_1",
      ownerUserId: "user_1",
      sourcePackId: "pack_1",
      targetId: "target_telegram",
      status: "queued",
      request: { options: {} },
      result: null,
      errorSummary: null,
      attemptCount: 0,
      maxAttempts: 3,
      nextAttemptAt: null,
      createdAt: "2026-05-07T00:00:00Z",
      updatedAt: "2026-05-07T00:00:00Z",
    };
    const fetchImpl = vi.fn(async (url: string, init?: RequestInit) => {
      if (url.endsWith("/events")) {
        return new Response(
          JSON.stringify([
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
          { status: 200 },
        );
      }

      return new Response(JSON.stringify(job), { status: init?.method === "POST" ? 201 : 200 });
    });
    const client = createExportClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_web_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const created = await client.createExportJob({
      id: "job_1",
      tenantId: "tenant_1",
      sourcePackId: "pack_1",
      targetId: "target_telegram",
      options: {},
    });
    const events = await client.listExportJobEvents("job_1");

    expect(created.status).toBe("queued");
    expect(events[0]?.message).toBe("job queued");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/export-jobs", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_web_secret",
      },
      body: JSON.stringify({
        id: "job_1",
        tenantId: "tenant_1",
        sourcePackId: "pack_1",
        targetId: "target_telegram",
        options: {},
      }),
    });
  });

  it("keeps redacted target config values from API responses", async () => {
    const target: ExportTarget = {
      id: "target_telegram",
      tenantId: "tenant_1",
      kind: "telegram",
      name: "Telegram",
      config: { botToken: "<redacted>" },
      isEnabled: true,
      createdAt: "2026-05-07T00:00:00Z",
      updatedAt: "2026-05-07T00:00:00Z",
    };
    const fetchImpl = vi.fn(async () => new Response(JSON.stringify(target), { status: 201 }));
    const client = createExportClient({
      baseUrl: "https://msm.example.test",
      authToken: "msm_pat_web_secret",
      fetchImpl,
    });

    const created = await client.createExportTarget({
      id: "target_telegram",
      tenantId: "tenant_1",
      kind: "telegram",
      name: "Telegram",
      config: { botToken: "123:secret" },
      isEnabled: true,
    });

    expect(created.config.botToken).toBe("<redacted>");
  });

  it("reads Telegram publication history with bearer auth", async () => {
    const fetchImpl = vi.fn(async (url: string) => {
      if (url.includes("/telegram-publications/telegram_pub_1")) {
        return new Response(
          JSON.stringify({
            id: "telegram_pub_1",
            packId: "pack_1",
            targetId: "target_telegram",
            jobId: "job_1",
            stickerSetName: "sample_pack_by_msm_bot",
            stickerSetUrl: "https://t.me/addstickers/sample_pack_by_msm_bot",
            stickerCount: 2,
            stickerType: "regular",
            createdAt: "2026-05-08T00:00:00Z",
            updatedAt: "2026-05-08T00:00:00Z",
          }),
          { status: 200 },
        );
      }

      return new Response(
        JSON.stringify([
          {
            id: "telegram_pub_1",
            packId: "pack_1",
            targetId: "target_telegram",
            jobId: "job_1",
            stickerSetName: "sample_pack_by_msm_bot",
            stickerSetUrl: "https://t.me/addstickers/sample_pack_by_msm_bot",
            stickerCount: 2,
            stickerType: "regular",
            createdAt: "2026-05-08T00:00:00Z",
            updatedAt: "2026-05-08T00:00:00Z",
          },
        ]),
        { status: 200 },
      );
    });
    const client = createExportClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_web_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    const publications = await client.listTelegramPublications("pack_1");
    const publication = await client.getTelegramPublication("telegram_pub_1");

    expect(publications[0]?.stickerSetUrl).toBe("https://t.me/addstickers/sample_pack_by_msm_bot");
    expect(publication.stickerSetName).toBe("sample_pack_by_msm_bot");
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/telegram-publications?packId=pack_1", {
      headers: {
        Authorization: "Bearer msm_pat_web_secret",
      },
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/telegram-publications/telegram_pub_1", {
      headers: {
        Authorization: "Bearer msm_pat_web_secret",
      },
    });
  });

  it("updates and deletes export targets with bearer auth", async () => {
    const target: ExportTarget = {
      id: "target_telegram",
      tenantId: "tenant_1",
      kind: "telegram",
      name: "Telegram",
      config: { botToken: "<redacted>" },
      isEnabled: false,
      createdAt: "2026-05-07T00:00:00Z",
      updatedAt: "2026-05-07T00:00:00Z",
    };
    const fetchImpl = vi.fn(async (_url: string, init?: RequestInit) => {
      if (init?.method === "DELETE") {
        return new Response(null, { status: 204 });
      }

      return new Response(JSON.stringify(target), { status: 200 });
    });
    const client = createExportClient({
      baseUrl: "https://msm.example.test/",
      authToken: "msm_pat_web_secret",
      fetchImpl: fetchImpl as typeof fetch,
    });

    await client.updateExportTarget({
      targetId: "target_telegram",
      name: "Telegram",
      config: { botToken: "123:secret" },
      isEnabled: false,
    });
    await client.deleteExportTarget("target_telegram");

    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/export-targets/target_telegram", {
      method: "PATCH",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer msm_pat_web_secret",
      },
      body: JSON.stringify({
        name: "Telegram",
        config: { botToken: "123:secret" },
        isEnabled: false,
      }),
    });
    expect(fetchImpl).toHaveBeenCalledWith("https://msm.example.test/api/v1/export-targets/target_telegram", {
      method: "DELETE",
      headers: {
        Authorization: "Bearer msm_pat_web_secret",
      },
    });
  });
});
