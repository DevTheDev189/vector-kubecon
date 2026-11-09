# Architectural map, monochrome design, and real GitHub sign-in

## Direction

Reconstruct the linked Salt Palace reference as native SVG, retaining the long east–west Halls A–E wing, north Halls 1–5 spine, south ballroom, foyers, and separate east-side meeting floors. Keep the SVG and live room bindings in code so every room can react to occupancy, selection, sensory preferences, and schedule changes. The reference is a low-resolution historical diagram, not a current survey: mark the result as a schematic and distinguish unmapped demo spaces. The user requested removal of the route line; show selected destinations without drawing walking routes.

Use a charcoal foundation with vibrant cyan, yellow, and magenta accents and restrained gradients (updated user preference). Cyan denotes live/open/quiet states, yellow denotes busy rooms, and magenta denotes surges/full rooms; labels and hatching preserve meaning independently of color. Use Geist Sans for the interface and Geist Mono for room numbers, time, and telemetry. Encode fullness with hatching, labels, and outlines. Keep touch controls at least 44px, support keyboard room selection, floor switching, zoom, reset, reduced motion, and responsive layouts.

## Implementation sequence

1. Create typed floor geometry and a rebuilt interactive SVG with reference-shaped halls, meeting room clusters, floor controls, pan/zoom, live room details, and clear destination highlighting. Correct the demo's 355 room floor in the display mapping without rewriting stored seed data silently.
2. Apply consistent grayscale tokens and fonts across the app. Make desktop map viewing larger while retaining the mobile schedule/peer flow. Keep all occupancy and selected-room views derived from Pinia state.
3. Require real GitHub OAuth for attendee data and actions. Remove impersonation, arbitrary-handle login, and `X-Vector-User`. Validate the OAuth state cookie and PKCE verifier, retrieve the authenticated identity from GitHub `/user` without seed fallback, and issue a signed HttpOnly SameSite=Lax cookie with Secure on HTTPS. Invalidate old demo sessions. Limit sync to the signed-in account, scope saved handshakes on the server, and clear private state on 401/logout.
4. Show an honest sign-in screen with a schematic map preview, session-only cookie explanation, configuration status, and sign-out. Keep public health and OAuth endpoints available. Protect demo mutation controls with an explicit GitHub admin allowlist.
5. Verify state reconciliation and session expiry, route protection, OAuth failure cases and a mocked complete provider callback, SVG room selection/floors/zoom, grayscale appearance, responsive layout, and offline shell behavior. A real GitHub login needs the operator's OAuth App credentials and browser consent.

## Operator setup needed

Create an OAuth App at https://github.com/settings/developers with homepage `https://vector.winslow.plus` and callback `https://vector.winslow.plus/api/auth/github/callback`. Set `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` in the host `.env` (never in frontend code or chat). Set `VECTOR_ADMIN_USERS` to comma-separated GitHub handles permitted to run surge/reset/check-in/audit controls. Existing `SESSION_SECRET` must remain private and contain at least 32 bytes. Restart the API after configuring it.

## Sources

- Floor-plan reference: the image linked by the user (`SPCC_floorplan.png`, Blogger/Googleusercontent).
- GitHub OAuth web flow/state/PKCE: https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps
- Geist Sans and Mono: https://vercel.com/font

## Status

Implementation complete; verification and review in progress. The user approved the architectural map and requested cyan/yellow/magenta accents and gradients over its dark foundation. Mobile admin controls support phone screen recordings. OAuth client credentials are absent from the current host; provider flow tests will use a local mock, with no production authentication bypass.

Local verification: ten Rust tests, nine frontend state/geometry tests, production build, Clippy, and three browser tests pass. OAuth credentials remain absent from the host; real provider consent needs operator setup. The known sample signing key was replaced locally with a random key; no secret is committed.

Follow-up: avatars render in full color, room search spans floors above the map, and quiet spaces use prominent cyan callouts. Supplemental data and create-only backfill are documented in `seed/README.md`. OAuth settings have since been configured privately on the host.
