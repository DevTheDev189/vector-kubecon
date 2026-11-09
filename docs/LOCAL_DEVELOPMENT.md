# Run and verify VECTOR

Run Elasticsearch 8.15 with `docker compose up -d elasticsearch`. Copy `.env.example` to `.env` and configure the desired integrations. Existing Elasticsearch data is retained; startup creates missing indices and seeds empty ones.

From the repository root:

```sh
cd frontend-pwa
npm ci
npm run build
cd ..
CARGO_BUILD_JOBS=1 cargo run -p backend-api
```

The Rust server serves both the PWA and `/api` on port 8080. `STATIC_DIR` overrides the frontend build directory. The default works from either the workspace root or `backend-api/`. Unknown `/api` paths return JSON 404 errors and missing `/assets` files return 404; app navigation paths return the SPA with status 200.

For frontend development, run `npm run dev` in `frontend-pwa/` alongside the API. Vite proxies `/api` to port 8080. Browser E2E tests use the production build served by Rust so they also exercise service-worker installation and offline shell caching.

```sh
CARGO_BUILD_JOBS=1 cargo test --workspace
CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets -- -D warnings
cd frontend-pwa
npm run build
npm test -- --maxWorkers=1
npx playwright install --with-deps chromium
npm run test:e2e
```

The browser tests expect the API and Elasticsearch to be running. `VECTOR_BASE_URL` defaults to `http://127.0.0.1:8080`; `ELASTICSEARCH_URL` defaults to `http://127.0.0.1:9200`. Run against a development instance: the test syncs Kelsey, triggers a surge, toggles sensory mode, and creates a handshake. It restores Kelsey's prior profile, deletes its handshake, and resets the surge afterward. Set `OPENAI_API_KEY=''` when starting the server to exercise deterministic fallbacks without spending credits. The Rust E2E test disables the AI key itself.

On the low-memory WSL host, `.cargo/config.toml` limits builds to one job. For the existing LLVM linker, this host also uses a temporary `RUSTC_WRAPPER=/tmp/vector-rustc-low-memory` that forwards compiler arguments and appends `-C link-arg=-Wl,--threads=1`. The extra swap does not replace these limits. Do not restart WSL or Tailscale during remote work.

The default QR phone link is `https://vector.winslow.plus`. Recommendations depend on the attendee's current GitHub DNA and available seats; Room 250A is one possible reroute destination. Quiet mode discovers the actual low-noise rooms. API health in the judge dock does not report Kubernetes replica counts or verify a Dash0 exporter.

The PWA caches its shell and built assets at installation. It never caches API responses; offline mode shows the shell and a reconnect control rather than stale personal conference data.
