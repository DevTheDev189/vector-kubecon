const CACHE = "vector-shell-v3";
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then(async (cache) => {
      const response = await fetch("/", { cache: "reload" });
      if (!response.ok) throw new Error("App shell unavailable");
      const html = await response.clone().text();
      const assets = [
        ...html.matchAll(/(?:src|href)="(\/assets\/[^"#?]+)"/g),
      ].map((match) => match[1]);
      await cache.addAll([
        ...new Set([...assets, "/icon.svg", "/manifest.webmanifest"]),
      ]);
      await cache.put("/", response);
    }),
  );
  self.skipWaiting();
});
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key.startsWith("vector-shell-") && key !== CACHE)
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});
self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (
    url.origin !== self.location.origin ||
    event.request.method !== "GET" ||
    url.pathname.startsWith("/api/")
  )
    return;
  // Navigations prefer the current app; offline shell never caches personal API data.
  if (event.request.mode === "navigate") {
    event.respondWith(
      fetch(event.request)
        .then((response) => {
          if (response.ok) {
            const copy = response.clone();
            event.waitUntil(
              caches.open(CACHE).then((cache) => cache.put("/", copy)),
            );
          }
          return response;
        })
        .catch(() => caches.match("/")),
    );
  } else if (url.pathname.startsWith("/assets/")) {
    event.respondWith(
      // These same-origin hashed files are public. The server's CORS Vary:
      // Origin header must not distinguish precache requests from module loads.
      caches.match(event.request, { ignoreVary: true }).then(
        (cached) =>
          cached ||
          fetch(event.request).then((response) => {
            if (response.ok) {
              const copy = response.clone();
              event.waitUntil(
                caches
                  .open(CACHE)
                  .then((cache) => cache.put(event.request, copy)),
              );
            }
            return response;
          }),
      ),
    );
  }
});
