# VECTOR

**Your day. Your people. Your pace.**

A mobile-first conference companion that turns attendee interests, session choices, room availability, and hallway connections into one experience.

[Try the live app](https://vector.winslow.plus) · [Local development](docs/LOCAL_DEVELOPMENT.md) · [OAuth setup](docs/GITHUB_OAUTH_SETUP.md)

## What it does

- Real GitHub OAuth sign-in with signed, HttpOnly session cookies.
- A recommended-day calendar linked to an architectural SVG venue map.
- Elasticsearch keyword/vector retrieval for sessions and similar attendees.
- Room updates and simulated crowd-surge rerouting through Server-Sent Events.
- Quiet-space discovery and decompression breaks.
- QR badges, hallway handshakes, and contextual follow-up drafts.

The demo has 49 seeded sessions, 24 mapped rooms, and 36 seeded profiles, plus real users. Supplemental attendees are labeled Demo. Crowd/noise readings and room placements are illustrative demo data; the map is not surveyed walking navigation.

## Architecture

Vue 3, TypeScript, Pinia, and Vite provide the mobile PWA. A Rust/Axum service serves both the frontend and API. Elasticsearch 8.15 stores session, profile, room, and connection data. The AI integration extracts interests and drafts follow-ups with deterministic fallbacks. The repository also contains a crowd simulator, container build, Kubernetes manifests, and CI.

## Run locally

Start Elasticsearch with `docker compose up -d elasticsearch`. Configure `.env` using the OAuth setup guide and a private session signing key, then:

```sh
npm ci --prefix frontend-pwa
npm run build --prefix frontend-pwa
CARGO_BUILD_JOBS=1 cargo run -p backend-api
```

The API/PWA runs on port 8080. Full instructions and isolated browser-test configuration are in [LOCAL_DEVELOPMENT.md](docs/LOCAL_DEVELOPMENT.md).

## Verification and evidence

- Rust integration tests cover OAuth, protected routes, room changes, surge/reset, search, handshakes, and non-destructive seed backfill.
- Frontend tests cover live reconciliation, session expiry, room geometry, and calendar/map selection.
- Playwright checks the desktop/mobile demo, keyboard dismissal, collapsed room search, QR/vault flow, and offline shell.
- [Room-audit performance exercise](docs/SRE_INCIDENT_REPORT.md): deliberate sequential baseline versus batched query, with local timing limits.
- [Sign-in timeout diagnosis](docs/SRE_SIGNIN_TIMEOUT.md): actual memory contention during final demo preparation and the recovery.

Dash0 exporter support and Kubernetes manifests are present, but code/configuration alone is not evidence of a live sponsor integration. Sponsor-specific claims and demo data provenance are documented explicitly.
