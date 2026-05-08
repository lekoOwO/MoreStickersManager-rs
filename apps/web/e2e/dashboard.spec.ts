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
  await page.getByTestId("sidebar-collapse").click();
  await expect(sidebar).toHaveAttribute("data-expanded", "true");
  await expect(sidebar).toContainText("MoreStickersManager");
  await expect(page.getByRole("tab")).toHaveCount(0);
});

test("pack layout does not force horizontal page overflow on narrow desktop", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "narrow-desktop", "narrow desktop assertion");

  await page.goto("/");

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
});
