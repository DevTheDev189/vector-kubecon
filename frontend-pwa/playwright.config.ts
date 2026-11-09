import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  workers: 1,
  timeout: 180_000,
  expect: { timeout: 30_000 },
  use: {
    baseURL: process.env.VECTOR_BASE_URL || "http://127.0.0.1:8080",
    viewport: { width: 1440, height: 1100 },
    reducedMotion: "reduce",
    screenshot: "only-on-failure",
    actionTimeout: 30_000,
    trace: "retain-on-failure",
    launchOptions: { args: ["--disable-dev-shm-usage"] },
  },
});
