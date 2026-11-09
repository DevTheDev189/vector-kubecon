# Sonar Gitar Repository Rules & Invariants

These invariants are strictly enforced across the VECTOR monorepo for automated PR review and code generation:

1. **Dash0 Observability on Route Handlers:**
   All public `axum` route handlers in `backend-api/src/routes/*.rs` MUST carry `#[tracing::instrument(skip_all, fields(http.route = ...))]` to ensure full distributed tracing visibility into Dash0.

2. **Strict RFC3339 UTC Timestamps:**
   All timestamps across domain models, Elasticsearch payloads, SSE events, and responses MUST use `chrono::DateTime<chrono::Utc>`. Naive or local timestamps are forbidden.

3. **No Unsafe Unwraps:**
   Route handlers and domain services MUST NOT use `.unwrap()` or `.expect()` on fallible I/O, HTTP requests, or JSON operations. All fallible operations must propagate errors via `?` into `ApiError`.

4. **Secure Signed Cookies:**
   All session cookies (`vector_sid`) MUST enforce `HttpOnly`, `SameSite::Lax`, proper path scoping, and HMAC-SHA256 signature verification.

5. **Accessibility & Decibel Preservation:**
   All talk and room queries MUST preserve `accessibility` metadata and `decibel_level` readings required by the Merge Forward sensory router and sensory haven engine.

## Attendee frontend conventions

Keep calendar selection and manual room lookup separate. Choosing another room must not change the selected agenda slot. Apply the same surge alternative to the agenda and map. Search and account controls stay collapsed until requested, with focus restoration and Escape dismissal. Keep conference dates and America/Denver times explicit; do not label seeded future sessions as live "now" events. Maintain full-color avatars, 44px touch controls, reduced-motion behavior, and no horizontal overflow on a 390px phone viewport.

Use the existing API error envelope and signed-cookie authentication. Never introduce impersonation/header bypasses, cache private API responses in the service worker, or include secrets in code, logs, PR descriptions, or screenshots. Seed backfill creates missing documents without replacing existing users or live room edits.
