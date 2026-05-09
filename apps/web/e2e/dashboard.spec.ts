import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem("msm.locale", "en");
    window.localStorage.removeItem("msm.pat");
  });
  await page.route("**/api/v1/packs?**", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify([
        {
          id: "MoreStickers:Telegram:Pack:dev_cats",
          title: "Development Cats",
          visibility: "public",
          stickerPack: {
            id: "MoreStickers:Telegram:Pack:dev_cats",
            title: "Development Cats",
            stickers: [{ id: "cat_1" }],
          },
          updatedAt: "2026-05-08T12:00:00Z",
        },
      ]),
    });
  });
  await page.route("**/api/v1/tenants/*/members", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify([
        {
          tenantId: "tenant_1",
          userId: "user_1",
          role: "admin",
          createdAt: "2026-05-09T00:00:00Z",
        },
        {
          tenantId: "tenant_1",
          userId: "user_2",
          role: "user",
          createdAt: "2026-05-09T00:00:00Z",
        },
      ]),
    });
  });
  await page.route("**/api/v1/tenants/*/members/*", async (route) => {
    await route.fulfill({
      contentType: "application/json",
      body: JSON.stringify({
        tenantId: "tenant_1",
        userId: "user_3",
        role: "admin",
        createdAt: "2026-05-09T00:00:00Z",
      }),
    });
  });
});

test("uses live API state instead of mock preview when env and PAT are present", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator('[data-testid="runtime-badge"]:visible')).toContainText("Live API");
  await expect(page.getByText(/Mock/)).toHaveCount(0);
  await expect(page.getByTestId("pack-section").getByRole("heading", { name: "Development Cats" }).first()).toBeVisible();
});

test("desktop has one navigation source and switches sections", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name === "mobile", "desktop navigation assertion");

  await page.goto("/");

  await expect(page.getByRole("tab")).toHaveCount(0);
  await page.getByRole("button", { name: "Export targets" }).click();

  await expect(page.getByRole("main").getByRole("heading", { name: "Export targets" }).first()).toBeVisible();
  await expect(page.getByTestId("pack-section")).toBeHidden();
});

test("desktop sidebar collapses and expands without duplicating top navigation", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name === "mobile", "desktop sidebar assertion");

  await page.goto("/");

  const sidebar = page.getByTestId("desktop-sidebar");
  await expect(sidebar).toHaveAttribute("data-expanded", "false");
  await expect(page.getByTestId("runtime-status")).not.toContainText("API");
  await expectRailControlsInside(sidebar, page.getByRole("button", { name: "MSM overview" }), page.getByTestId("sidebar-collapse"));
  await page.getByTestId("sidebar-collapse").click();
  await expect(sidebar).toHaveAttribute("data-expanded", "true");
  const brand = page.getByTestId("sidebar-brand").getByText("MoreStickersManager");
  await expect(brand).toBeVisible();
  await expect(async () => {
    const isClipped = await brand.evaluate((element) => element.scrollWidth > element.clientWidth + 1);
    expect(isClipped).toBe(false);
  }).toPass();
  await expect(page.getByRole("tab")).toHaveCount(0);
});

test("PAT scopes are selectable controls instead of a raw text field", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("button", { name: "PAT" }).click();
  const dialog = page.getByRole("dialog", { name: "Personal Access Tokens" });

  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("checkbox", { name: /Read packs/ })).toBeChecked();
  await expect(dialog.getByRole("checkbox", { name: /Manage PATs/ })).toBeChecked();
  await expect(dialog.getByRole("checkbox", { name: /Manage tenant members/ })).toBeChecked();
  await expect(dialog.getByRole("textbox", { name: "Scopes" })).toHaveCount(0);
});

test("tenant admin workspace lists members and exposes role assignment", async ({ page }, testInfo) => {
  await page.goto("/");

  if (testInfo.project.name === "mobile") {
    await page.getByRole("button", { name: "Navigation" }).click();
  }
  await page.getByRole("button", { name: "Tenant admin" }).first().click();

  await expect(page.getByRole("main").getByRole("heading", { name: "Tenant admin" }).first()).toBeVisible();
  const tenantAdminSection = page.getByTestId("tenant-admin-section");
  await expect(tenantAdminSection.locator("p:visible", { hasText: "user_1" }).first()).toBeVisible();
  await page.getByLabel("Member user ID").fill("user_3");
  await page.getByLabel("Member role").first().selectOption("admin");
  await page.getByRole("button", { name: "Set member role" }).click();
  await expect(tenantAdminSection.locator("p:visible", { hasText: "user_2" }).first()).toBeVisible();
});

test("zh-TW chrome translates the fixed dashboard and access-token labels", async ({ page }, testInfo) => {
  await page.goto("/");

  await page.getByRole("button", { name: "Language" }).click();
  await expect(page.getByRole("main").getByRole("heading", { name: "貼圖包" }).first()).toBeVisible();
  if (testInfo.project.name === "mobile") {
    await page.getByRole("button", { name: "導覽" }).click();
  }
  await expect(page.getByRole("button", { name: "匯出目標" }).first()).toBeVisible();

  await page.getByRole("button", { name: "PAT" }).click();
  const dialog = page.getByRole("dialog", { name: "個人存取權杖" });

  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText("權限範圍");
  await expect(dialog).toContainText("讀取貼圖包");
  await expect(dialog).not.toContainText("Personal Access Tokens");
  await expect(dialog).not.toContainText("Scopes");
  await dialog.getByRole("button", { name: "關閉" }).click();

  await page.getByRole("button", { name: "開啟貼圖包匯入視窗" }).click();
  const importDialog = page.getByRole("dialog", { name: "匯入貼圖包" });
  await expect(importDialog).toBeVisible();
  await expect(importDialog).toContainText("匯入貼圖包 JSON");
  await expect(importDialog.getByRole("button", { name: "關閉" })).toBeVisible();
});

test("pack layout does not force horizontal page overflow on narrow desktop", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "narrow-desktop", "narrow desktop assertion");

  await page.goto("/");

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
});

async function expectRailControlsInside(sidebar: import("@playwright/test").Locator, ...controls: import("@playwright/test").Locator[]) {
  const sidebarBox = await sidebar.boundingBox();
  expect(sidebarBox).not.toBeNull();

  for (const control of controls) {
    const controlBox = await control.boundingBox();
    expect(controlBox).not.toBeNull();
    expect(controlBox!.x).toBeGreaterThanOrEqual(sidebarBox!.x + 8);
    expect(controlBox!.x + controlBox!.width).toBeLessThanOrEqual(sidebarBox!.x + sidebarBox!.width - 8);
  }
}
