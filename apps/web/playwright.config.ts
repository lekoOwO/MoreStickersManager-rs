import { defineConfig, devices } from "@playwright/test";
import { fileURLToPath } from "node:url";

const webRoot = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  expect: {
    timeout: 5_000,
  },
  use: {
    baseURL: "http://127.0.0.1:4173",
    channel: "msedge",
    trace: "on-first-retry",
  },
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 4173 --strictPort",
    cwd: webRoot,
    env: {
      VITE_MSM_API_BASE_URL: "http://127.0.0.1:4173",
      VITE_MSM_PAT: "msm_pat_e2e",
      VITE_MSM_USER_ID: "user_1",
    },
    reuseExistingServer: !process.env.CI,
    url: "http://127.0.0.1:4173",
  },
  projects: [
    {
      name: "desktop",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1366, height: 900 },
      },
    },
    {
      name: "narrow-desktop",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1100, height: 820 },
      },
    },
    {
      name: "mobile",
      use: {
        ...devices["Pixel 7"],
      },
    },
  ],
});
