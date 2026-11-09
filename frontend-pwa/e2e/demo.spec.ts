import { expect, test } from "@playwright/test";

test.beforeEach(async ({ context }) => {
  await context.route("https://fonts.googleapis.com/**", (route) =>
    route.abort(),
  );
  await context.route("https://fonts.gstatic.com/**", (route) => route.abort());
});

test("desktop and mobile attendee demo", async ({ page, context, request }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const elastic = process.env.ELASTICSEARCH_URL || "http://127.0.0.1:9200";
  const originalResponse = await request.get(
    `${elastic}/builders/_doc/kelseyhightower`,
  );
  expect(originalResponse.ok()).toBe(true);
  const original = await originalResponse.json();
  let handshakeId = "";
  try {
    await page.goto("/");
    await expect(page.locator(".talk-card")).toBeVisible();
    await expect(page.locator(".map-room")).toHaveCount(12);
    await page
      .locator(".judge-presets")
      .getByRole("button", { name: "@kelseyhightower", exact: true })
      .click();
    await expect(page.locator(".identity-button")).toContainText(
      "@kelseyhightower",
      { timeout: 90_000 },
    );
    const trigger = page.getByRole("button", { name: "Trigger crowd surge" });
    await expect(trigger).toBeEnabled({ timeout: 90_000 });
    const surgeResponse = page.waitForResponse(
      (r) => r.url().endsWith("/api/surge") && r.request().method() === "POST",
    );
    await trigger.click();
    const response = await surgeResponse;
    expect(response.ok()).toBe(true);
    const alert = (await response.json()).surge.data;
    expect(alert.recommended_alternative.room_id).not.toBe("ballroom-b");
    expect(alert.recommended_alternative.occupancy_ratio).toBeLessThan(0.98);
    await expect(page.locator(".surge-banner")).toBeVisible();
    await expect(page.locator(".talk-card")).toContainText(
      alert.recommended_alternative.title,
    );
    await expect(page.locator(".reroute-path")).toHaveCount(1);
    await page
      .getByRole("button", { name: "Try quiet mode", exact: true })
      .click();
    await expect(page.locator(".haven-banner")).toBeVisible();
    await page
      .getByRole("button", { name: "Open QR badge", exact: true })
      .click();
    await expect(page.locator(".qr-badge img")).toBeVisible();
    await page.getByLabel("Their GitHub handle").fill("zircon-dev");
    const handshakeResponse = page.waitForResponse(
      (r) =>
        r.url().endsWith("/api/handshake/connect") &&
        r.request().method() === "POST",
    );
    await page
      .getByRole("button", { name: "Connect & draft follow-up" })
      .click();
    const connection = await handshakeResponse;
    expect(connection.ok()).toBe(true);
    handshakeId = (await connection.json()).handshake.handshake_id;
    await expect(
      page.getByRole("heading", { name: "Good conversations, kept." }),
    ).toBeVisible();
    await expect(page.locator(".connection-card").first()).toBeVisible();
    await page.getByRole("button", { name: "Close dialog" }).click();
    await page
      .getByRole("button", { name: "Reset surge", exact: true })
      .click();
    await expect(page.locator(".surge-banner")).toHaveCount(0);
    await page
      .getByRole("button", { name: "Quiet mode on", exact: true })
      .click();
    await expect(page.locator(".haven-banner")).toHaveCount(0);
    await page.setViewportSize({ width: 390, height: 844 });
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await expect(page.locator(".judge-dock")).toBeHidden();
    await expect(page.locator(".spatial-map")).toBeVisible();
    await page.goto("/?peer=zircon-dev&pin=4829");
    await expect(page.getByLabel("Their GitHub handle")).toHaveValue(
      "zircon-dev",
    );
    await expect(page.getByLabel("Their PIN")).toHaveValue("4829");
    await page.keyboard.press("Escape");
    await expect(page.locator("dialog")).toHaveCount(0);
    expect(errors).toEqual([]);
  } finally {
    if (handshakeId)
      await request.delete(
        `${elastic}/handshakes/_doc/${handshakeId}?refresh=true`,
      );
    await request.post("/api/surge/reset", { data: {} });
    await request.put(`${elastic}/builders/_doc/kelseyhightower?refresh=true`, {
      data: original._source,
    });
  }
});

test("installed shell reloads offline without caching API responses", async ({
  page,
  context,
}) => {
  await page.goto("/");
  await expect(page.locator(".talk-card")).toBeVisible();
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready;
    const keys = await caches.keys();
    const cache = await caches.open(
      keys.find((key) => key.startsWith("vector-shell-"))!,
    );
    const urls = (await cache.keys()).map(
      (request) => new URL(request.url).pathname,
    );
    if (
      !urls.some((url) => url.endsWith(".js")) ||
      !urls.some((url) => url.endsWith(".css"))
    )
      throw new Error("First install did not cache the built app");
    if (urls.some((url) => url.startsWith("/api/")))
      throw new Error("Personal API data cached");
  });
  await context.setOffline(true);
  await page.reload({ waitUntil: "domcontentloaded" });
  await expect(page.locator(".wordmark")).toContainText("VECTOR");
  await expect(
    page.getByRole("button", { name: "Reconnect", exact: true }),
  ).toBeVisible();
  await context.setOffline(false);
});
