# Run and verify VECTOR

Start Elasticsearch 8.15 with `docker compose up -d elasticsearch`. Copy `.env.example` to `.env`, generate a private signing key (`openssl rand -hex 32`), and follow [GitHub OAuth setup](GITHUB_OAUTH_SETUP.md). Existing Elasticsearch data is retained; startup creates missing indices and seeds empty ones.

```sh
cd frontend-pwa
npm ci
npm run build
cd ..
CARGO_BUILD_JOBS=1 cargo run -p backend-api
```

The Rust server serves the PWA and `/api` on port 8080. `STATIC_DIR` overrides the build directory; the default works from the workspace root or `backend-api/`. Unknown API routes return JSON 404s, missing assets return 404, and SPA navigation returns HTML with status 200.

Real GitHub OAuth is required for attendee APIs. The app has no impersonation route or identity-header bypass. Public endpoints are health, auth configuration status, OAuth start/callback, and static assets. `VECTOR_ADMIN_USERS` grants signed-in accounts access to surge/reset/check-in/audit controls. The simulator now needs a real administrator session in `VECTOR_SESSION_COOKIE`; treat it as a credential and do not store it in source control.

For Vite development, run `npm run dev` alongside the API. Configure a separate local OAuth App and `PUBLIC_BASE_URL` to match the browser origin (usually `http://127.0.0.1:5173`), with callback `/api/auth/github/callback`. Vite proxies API requests to port 8080. The backend checks the Origin of browser mutations against `PUBLIC_BASE_URL`.

```sh
CARGO_BUILD_JOBS=1 cargo test --workspace
CARGO_BUILD_JOBS=1 cargo clippy --workspace --all-targets -- -D warnings
cd frontend-pwa
npm run build
npm test -- --maxWorkers=1
npx playwright install --with-deps chromium
```

Rust tests exercise an entire OAuth callback against a local provider mock, then use the actual protected HTTP routes and Elasticsearch. They also check state/PKCE failures, old/expired cookies, arbitrary-header rejection, account ownership, and admin restrictions. AI calls are disabled for those tests.

Browser tests run against a **dedicated development test server**. They sign test fixture cookies with an explicit test-only key; this does not add an authentication bypass to the application. Never run the normal public server with this key.

In one terminal, from the repository root:

```sh
PORT=18082 PUBLIC_BASE_URL=http://127.0.0.1:18082 \
SESSION_SECRET=vector-browser-test-only-signing-secret-32bytes \
VECTOR_ADMIN_USERS=kelseyhightower \
GITHUB_CLIENT_ID=test-ui-client GITHUB_CLIENT_SECRET=test-ui-secret \
OPENAI_API_KEY='' cargo run -p backend-api
```

In a second terminal:

```sh
cd frontend-pwa
VECTOR_BASE_URL=http://127.0.0.1:18082 \
VECTOR_TEST_SESSION_SECRET=vector-browser-test-only-signing-secret-32bytes \
npm run test:e2e
```

The tests expect Elasticsearch at `http://127.0.0.1:9200` (override `ELASTICSEARCH_URL` if needed). They reset their attendee fixture, temporarily adjust a room-noise reading, trigger/reset surges, and create/delete a handshake. They restore the original attendee profile. Tests cover anonymous access, authenticated desktop/mobile flow, map floors/zoom/selection, SSE updates, mobile recording controls, sign-out, QR deep links, and offline shell behavior. API responses are never cached; self-hosted fonts and app assets are cached for offline installation.

On this WSL host, `.cargo/config.toml` limits builds to one job. The host also uses `RUSTC_WRAPPER=/tmp/vector-rustc-low-memory`, which forwards compiler arguments and appends `-C link-arg=-Wl,--threads=1`. Keep these limits despite the added swap. Do not restart WSL or Tailscale during remote work.

The rebuilt map follows the supplied historical Salt Palace diagram. It is an architectural schematic with approximate demo-room placements, not surveyed pedestrian or accessibility navigation. A prominent cross-floor text search and quiet-space discovery list provide usable phone targets for small meeting-room shapes. Recommendations depend on the attendee's verified GitHub profile and available seats.

Additional demo fixtures are backfilled into existing databases without overwriting records. See [seed data](../seed/README.md).
