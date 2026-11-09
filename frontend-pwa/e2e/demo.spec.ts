import { expect, test } from "@playwright/test";

import { createHmac } from "node:crypto";

async function signInFixture(
  context: import("@playwright/test").BrowserContext,
) {
  const secret = process.env.VECTOR_TEST_SESSION_SECRET;
  if (!secret || secret.length < 32)
    throw new Error(
      "Start a dedicated test server with SESSION_SECRET and VECTOR_TEST_SESSION_SECRET set to the same test-only value",
    );
  const now = Math.floor(Date.now() / 1000);
  const payload = Buffer.from(
    JSON.stringify({
      username: "kelseyhightower",
      github_id: 110292,
      iat: now,
      exp: now + 3600,
      auth_version: 1,
    }),
  ).toString("base64url");
  const signature = createHmac("sha256", secret)
    .update(payload)
    .digest("base64url");
  await context.addCookies([
    {
      name: "vector_sid",
      value: `${payload}.${signature}`,
      url: process.env.VECTOR_BASE_URL || "http://127.0.0.1:8080",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

test("anonymous visitors see the map preview and real GitHub sign-in", async ({
  page,
  request,
}) => {
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Find your place. Meet your people." }),
  ).toBeVisible();
  await expect(
    page.locator(".signin-preview .architectural-map"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Sign in with GitHub" }),
  ).toHaveAttribute("href", "/api/auth/github");
  await expect(page.locator(".talk-card")).toHaveCount(0);
  expect((await request.get("/api/state")).status()).toBe(401);
  expect(
    (
      await request.post("/api/auth/impersonate", {
        data: { username: "kelseyhightower" },
      })
    ).status(),
  ).toBe(404);
  await page.setViewportSize({ width: 390, height: 844 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
});

test("desktop and mobile attendee demo", async ({ page, context }) => {
  await signInFixture(context);
  const request = context.request;
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  const elastic = process.env.ELASTICSEARCH_URL || "http://127.0.0.1:9200";
  const originalResponse = await request.get(
    `${elastic}/builders/_doc/kelseyhightower`,
  );
  expect(originalResponse.ok()).toBe(true);
  const original = await originalResponse.json();
  await request.put(`${elastic}/builders/_doc/kelseyhightower?refresh=true`, {
    data: {
      ...original._source,
      sensory_prefs: {
        ...original._source.sensory_prefs,
        sensory_mode_enabled: false,
      },
    },
  });
  const initialRooms = await (await request.get("/api/rooms")).json();
  const originalNoise = initialRooms.rooms.find(
    (room: { room_id: string }) => room.room_id === "room-250a",
  ).decibel_level;
  let handshakeId = "";
  try {
    await page.goto("/");
    await expect(
      page.getByRole("heading", { name: "Your recommended day", exact: true }),
    ).toBeVisible();
    expect(await page.locator(".agenda-row").count()).toBeGreaterThanOrEqual(5);
    expect(
      await page
        .locator(".day-calendar")
        .evaluate(
          (calendar) =>
            !!(
              calendar.compareDocumentPosition(
                document.querySelector(".architectural-map")!,
              ) & Node.DOCUMENT_POSITION_FOLLOWING
            ),
        ),
    ).toBe(true);
    expect(
      await page
        .locator(".architectural-map")
        .evaluate(
          (map) =>
            !!(
              map.compareDocumentPosition(
                document.querySelector(".builder-identity")!,
              ) & Node.DOCUMENT_POSITION_FOLLOWING
            ),
        ),
    ).toBe(true);
    await expect(
      page.getByRole("searchbox", { name: "Find your room" }),
    ).toHaveCount(0);
    await expect(page.locator(".account-options")).not.toHaveAttribute(
      "open",
      "",
    );
    const dayState = await (await request.get("/api/state")).json();
    const firstTalk = dayState.schedule.slots[0].primary_talk;
    await page.locator('.agenda-row[data-slot-index="0"] button').click();
    await page
      .getByRole("button", { name: "View on map", exact: true })
      .click();
    if (firstTalk)
      await expect(
        page.locator(`.map-room[data-room-id="${firstTalk.room_id}"]`),
      ).toHaveAttribute("aria-pressed", "true");
    await page.locator('.agenda-row[data-slot-index="2"] button').click();
    await page
      .getByRole("button", { name: "01 First floor", exact: true })
      .click();
    await expect(
      page.locator('.map-room[data-room-id="ballroom-b"]'),
    ).toBeVisible();
    await expect(page.locator(".signed-in-account")).toContainText(
      "@kelseyhightower",
    );
    await page.getByRole("button", { name: "Zoom in", exact: true }).click();
    await expect(page.locator(".map-tools")).toContainText("150%");
    await page
      .getByRole("button", { name: "Reset map view", exact: true })
      .click();
    await page
      .getByRole("button", { name: "02 Second floor", exact: true })
      .click();
    await expect(
      page.locator('.map-room[data-room-id="room-250a"]'),
    ).toBeVisible();
    await page.locator('.map-room[data-room-id="room-250a"]').click();
    await expect(page.locator(".map-selection")).toContainText("Room 250A");
    const search = page.getByRole("searchbox", { name: "Find your room" });
    await page
      .getByRole("button", { name: "Find your room", exact: true })
      .click();
    await expect(search).toBeFocused();
    await search.fill("355a");
    await page.keyboard.press("Escape");
    await expect(search).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: "Find your room", exact: true }),
    ).toBeFocused();
    await page
      .getByRole("button", { name: "Find your room", exact: true })
      .click();
    await search.fill("355a");
    await page
      .locator(".room-search-results")
      .getByRole("button", { name: /Room 355A/ })
      .click();
    await expect(page.locator(".map-selection")).toContainText("Room 355A");
    await expect(
      page.locator('.map-room[data-room-id="room-355a-quiet"]'),
    ).toBeVisible();
    await expect(page.locator(".quiet-map-pin")).toHaveCount(1);
    await page.getByRole("button", { name: /Quiet spaces/ }).click();
    await expect(page.locator(".room-search-results button")).toHaveCount(5);
    await page
      .locator(".room-search-results")
      .getByRole("button", { name: /Room 204B/ })
      .click();
    await expect(page.locator(".map-selection")).toContainText("Room 204B");
    await expect(page.locator(".avatar").first()).toHaveCSS("filter", "none");
    await page
      .getByRole("button", { name: "Find your room", exact: true })
      .click();
    await search.fill("250a");
    await page
      .locator(".room-search-results")
      .getByRole("button", { name: /Room 250A/ })
      .click();
    const noise = await request.post("/api/rooms/checkin", {
      data: { room_id: "room-250a", delta: 0, decibel_reading: 61.5 },
    });
    expect(noise.ok()).toBe(true);
    await expect(page.locator(".map-selection")).toContainText("61.5 dB");
    await page
      .getByRole("button", { name: "03 Third floor", exact: true })
      .click();
    await expect(
      page.locator('.map-room[data-room-id="room-355e"]'),
    ).toBeVisible();
    await page
      .getByRole("button", { name: "01 First floor", exact: true })
      .click();
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
    await expect(page.locator(".agenda-row.selected")).toContainText(
      alert.recommended_alternative.title,
    );
    await page.locator(".session-details > summary").click();
    await expect(page.locator(".agenda-list")).toBeVisible();
    await expect(page.locator(".talk-card")).toContainText(
      alert.recommended_alternative.title,
    );
    await page.locator(".session-details > summary").click();
    await expect(page.locator(".reroute-path")).toHaveCount(0);
    await page
      .getByRole("button", { name: "Try quiet mode", exact: true })
      .click();
    await expect(page.locator(".haven-banner")).toBeVisible();
    await page.getByRole("button", { name: "Show quiet space" }).click();
    await expect(page.locator(".map-selection")).toContainText("Room 204B");
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
    const selectedIndex = await page
      .locator(".agenda-row.selected")
      .getAttribute("data-slot-index");
    await page
      .getByRole("button", { name: "Find your room", exact: true })
      .click();
    await search.fill("203a");
    await expect(page.locator(".room-search-results")).toBeVisible();
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    await page
      .locator(".room-search-results")
      .getByRole("button", { name: /Room 203A/ })
      .click();
    await expect(page.locator(".map-selection")).toContainText("Room 203A");
    await expect(page.locator(".agenda-row.selected")).toHaveAttribute(
      "data-slot-index",
      selectedIndex!,
    );
    await page
      .getByRole("button", { name: "Back to my session", exact: true })
      .click();
    await expect(
      page.getByRole("button", { name: "Back to my session", exact: true }),
    ).toHaveCount(0);
    const mobileControls = page.locator(".mobile-demo-controls");
    await mobileControls.locator("summary").click();
    await mobileControls
      .getByRole("button", { name: "Trigger surge", exact: true })
      .click();
    await expect(page.locator(".surge-banner")).toBeVisible();
    await mobileControls.locator("summary").click();
    await mobileControls
      .getByRole("button", { name: "Reset surge", exact: true })
      .click();
    await expect(page.locator(".surge-banner")).toHaveCount(0);
    await page.goto("/?peer=zircon-dev&pin=4829");
    await expect(page.getByLabel("Their GitHub handle")).toHaveValue(
      "zircon-dev",
    );
    await expect(page.getByLabel("Their PIN")).toHaveValue("4829");
    await page.keyboard.press("Escape");
    await expect(page.locator("dialog")).toHaveCount(0);
    expect(errors).toEqual([]);
    await request.post("/api/rooms/checkin", {
      data: { room_id: "room-250a", delta: 0, decibel_reading: originalNoise },
    });
    await page.locator(".account-options > summary").click();
    await page.getByRole("button", { name: "Sign out", exact: true }).click();
    await expect(page.locator(".signin-copy")).toBeVisible();
    await expect(page.locator(".talk-card")).toHaveCount(0);
  } finally {
    if (handshakeId)
      await request.delete(
        `${elastic}/handshakes/_doc/${handshakeId}?refresh=true`,
      );

    await request.put(`${elastic}/builders/_doc/kelseyhightower?refresh=true`, {
      data: original._source,
    });
  }
});

test("installed shell reloads offline without caching API responses", async ({
  page,
  context,
}) => {
  await signInFixture(context);
  await page.goto("/");
  await expect(page.locator(".agenda-list")).toBeVisible();
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
  await expect(page.locator(".signin-copy")).toBeVisible();
  await expect(page.locator(".talk-card")).toHaveCount(0);
  await context.setOffline(false);
});
