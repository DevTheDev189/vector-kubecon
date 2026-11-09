# VECTOR submission copy

## Project title

VECTOR — Your day. Your people. Your pace.

## Repository

https://github.com/DevTheDev189/vector-kubecon

## Demo video

Record and upload a video under three minutes, then paste its public or unlisted URL into the form. Test the URL while signed out. This field must be supplied by the maintainer; no video has been uploaded by the coding agent.

## Live demo

https://vector.winslow.plus

## Tracks

Verified strong fit: Elasticsearch and Merge Forward. Select Gitar, Dash0, or Radar only if the maintainer supplies genuine usage evidence. Configuring repository rules, local trace IDs, or Kubernetes manifests alone does not demonstrate the sponsor workflows. The current host has no Dash0 exporter credentials configured, no Gitar bot comments/reviews have been verified, and the duplicate k3d stack was stopped to recover memory for the live demo.

## What conference problem are you patching?

Conference attendees have to choose between overlapping sessions, navigate unfamiliar rooms, react when a room fills up, and find people with shared interests. Static schedules do not adapt to crowding or sensory needs, while valuable hallway conversations are easy to miss or forget. VECTOR brings the recommended day, venue context, quieter spaces, and relevant people into one mobile experience so attendees can spend less time coordinating and more time participating.

## What did you build and how does it work?

VECTOR is a mobile-first conference companion. Attendees sign in with real GitHub OAuth, and their public profile and repositories help build an interest profile. An AI integration extracts technical interests and drafts contextual follow-ups, with deterministic fallbacks for demo resilience.

A Rust/Axum API uses Elasticsearch as the core store for sessions, attendee profiles, room state, and connections. Hybrid keyword/vector retrieval recommends sessions and similar builders, while a schedule solver avoids overlapping recommendations. The Vue/Pinia frontend presents a compact recommended-day calendar, an interactive SVG venue map, room lookup, and nearby peers.

Room updates arrive through Server-Sent Events. A simulated crowd surge fills a room and triggers an open-session alternative; quiet mode surfaces low-noise, lightly occupied spaces. QR badges and hallway handshakes save meeting context and produce LinkedIn/email drafts that the attendee can use afterward.

The demo contains 49 seeded sessions, 24 mapped rooms, and 36 seeded profiles, plus real OAuth users. Additional attendees are explicitly labeled Demo, and crowd/noise readings are simulated rather than live venue sensors.

## How did you use Elasticsearch?

Elasticsearch is VECTOR's core data layer, storing conference talks, builder profiles, room telemetry, and saved handshakes. Talk retrieval combines BM25 multi_match keyword queries with kNN over 1536-dimensional embeddings. Capacity, time-window, and sensory/accessibility criteria help shape recommendations. Vector similarity and shared technical interests also power hallway matching.

When a room fills up, the API updates room/talk state and searches for an open alternative. Quiet-space discovery queries unscheduled rooms below the demo's noise and occupancy thresholds. We also replaced a deliberately sequential room-audit exercise with one batched _mget query; the measured local median fell from 173.843 ms to 16.537 ms. The code history and measurement limits are documented in docs/SRE_INCIDENT_REPORT.md.

This is a search/recommendation workflow; we did not build a RAG chatbot.

## Merge Forward: How does your project make conferences more inclusive and accessible?

VECTOR helps attendees who find crowds, noise, unfamiliar venues, or unstructured networking difficult to navigate. Quiet mode identifies unscheduled rooms with low simulated noise and occupancy, and the map highlights them with text labels as well as color. The schedule supports decompression breaks, while room data preserves wheelchair-access and captioning metadata where provided.

The mobile interface uses large touch targets, keyboard-operable controls, readable room-search results, full-color avatars, and reduced-motion support. A compact recommended-day view reduces the amount of information attendees must manage at once. Shared-interest matches, QR handshakes, and contextual follow-up drafts help make networking more approachable. Room data is demonstrative and does not establish a venue accessibility certification.

## Gitar — only if verified usage is supplied

Implemented review conventions in .gitar/review/vector-conventions.md (with the original conventions also in .gitar/rules.md) covering route instrumentation, RFC3339 UTC timestamps, error handling, signed-cookie protections, and preservation of accessibility/noise metadata. Features landed incrementally through PRs #1–#7 and automated CI; the final UX work is PR #8. No actual Gitar review, CI diagnosis, or auto-apply finding has been verified. Do not describe GitHub Actions checks as Gitar reviews.

Useful PR links: https://github.com/DevTheDev189/vector-kubecon/pull/2, https://github.com/DevTheDev189/vector-kubecon/pull/4, https://github.com/DevTheDev189/vector-kubecon/pull/7

## Dash0 — only if verified usage is supplied

The code includes instrumented Axum handlers, an OTLP trace exporter option, structured application logging, and instruments for room occupancy/noise, surge reroutes, query latency, token counts, and SSE clients. However, the running demo has no Dash0 endpoint/token configured, and exported metrics, RUM, a dashboard/SLO, and Agent0/MCP findings have not been verified. Do not claim these as completed sponsor evidence or describe fallback UUIDs as verified Dash0 traces.

## Radar — only if verified usage is supplied

The repository contains Kubernetes deployment manifests and the host had a k3d cluster. During final preparation, the duplicate cluster and unrelated Docker workloads competed with the primary Elasticsearch for the host's limited RAM, causing OAuth/profile timeouts. With the maintainer's authorization, the duplicate stack was stopped; profile reads and authenticated state recovered. This was a local diagnosis using logs/process inspection and HTTP probes, not a verified Radar UI/MCP/AI investigation. Do not claim otherwise.

## Optional product feedback

Leave feedback blank unless it reflects the maintainer's own experience with each tool. Do not invent comments about sponsor products that were not used.

## Two-and-a-half-minute recording outline

- 0:00–0:15: State the attendee problem: sessions, crowding, quiet space, and meaningful connections.
- 0:15–0:45: Show the signed-in recommended-day calendar. Select a session and view it on the map.
- 0:45–1:10: Open the compact room finder, then show quiet-space discovery.
- 1:10–1:40: Use admin demo controls to simulate a surge; show the alternative recommendation and map response. Explain that crowd/noise data is simulated.
- 1:40–2:10: Show the attendee profile, similar people, QR badge, and saved follow-up drafts.
- 2:10–2:35: Explain the Rust API, Elasticsearch hybrid retrieval, and SSE updates; end on the product's benefit and live/repository links.

Do not include credential settings or browser developer tools containing secrets in the recording. Keep the final video under three minutes and allow time for upload processing.

## Finish within the deadline

1. Next 10 minutes: confirm a fresh phone sign-in, one full rehearsal, and freeze the app.
2. Next 25 minutes: record one clean take, then a second only if necessary.
3. Next 20 minutes: upload the video and paste the project/track copy into the form.
4. Submit with at least 30 minutes remaining. Test the public/unlisted video, repository, and HTTPS live link from a signed-out browser. Keep remaining time as upload/submission buffer rather than adding features.
