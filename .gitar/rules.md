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
