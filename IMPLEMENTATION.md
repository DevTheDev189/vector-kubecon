# VECTOR (`vector.winslow.plus`) — Autonomous Agent Master Implementation Plan
**Target Architecture:** ~7,000 LOC Production Monorepo (Rust `axum` + Elasticsearch 8.15 Hybrid RRF + OpenAI GPT-6.1 Sol + Dash0 OpenTelemetry + Radar Kubernetes + Sonar Gitar + Vue 3 / TypeScript / Tailwind CSS Spatial PWA)

---

## 0. AUTONOMOUS AGENT EXECUTION LOOP PROTOCOL (READ FIRST)

You are an autonomous principal engineer executing this specification from top to bottom. You MUST obey the following state-machine loop strictly:

1. **Locate Next Task:** Scan this `IMPLEMENTATION.md` file for the first unchecked `[ ]` item in the current Phase.
2. **Write Complete Production Code:** Never write `// TODO`, `unimplemented!()`, or placeholder mocks. Write complete, well-structured, idiomatic Rust and Vue 3 TypeScript code that fulfills the exact file specification and target depth.
3. **Verify Immediately:** After creating or modifying the file(s) in a step, run the compiler/linter/test command (`cargo check`, `cargo test`, `npm run build`, etc.) and fix any warnings or errors before proceeding.
4. **Update State In-Place:** Once a task builds and passes verification, edit this `IMPLEMENTATION.md` file directly to change `[ ]` to `[x]` for that item.
5. **Phase Gate & Git PR Workflow (Sonar / Gitar $4,000 Track Requirement):**
   - Never push directly to `main` after initial bootstrap.
   - Each Phase MUST be developed on its designated feature branch (`feat/phase-1-...` through `feat/phase-6-...`).
   - When all `[x]` tasks and the **Phase Verification Gate** in a Phase are complete, stage all files (including the updated `IMPLEMENTATION.md`), commit with a conventional commit message, push the branch to `origin`, and open/merge the Pull Request via `gh pr create` so **Sonar Gitar** reviews every PR incrementally throughout the day.

---

## PHASE 1: Workspace Spine, Gitar Rules, K8s Manifests & Telemetry Core (~950 LOC)
**Branch:** `feat/phase-1-workspace-and-infra`
**Goal:** Establish the Rust Cargo workspace, Sonar Gitar repository rules, GitHub Actions CI, local Elasticsearch + Kubernetes manifests for Radar, and the foundational Rust `axum` configuration, error handling, and Dash0 OpenTelemetry pipeline.

### 1.1 Repository Root & Sonar Gitar Configuration
- [x] **Create `.gitignore`**
  - Ignore `/target`, `/frontend-pwa/node_modules`, `/frontend-pwa/dist`, `.env`, `*.log`, `.DS_Store`.
- [x] **Create `.env.example`**
  - Define documented keys: `PORT=8080`, `PUBLIC_BASE_URL=https://vector.winslow.plus`, `SESSION_SECRET`, `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `OPENAI_API_KEY`, `OPENAI_MODEL=gpt-6.1-sol`, `OPENAI_EMBEDDING_MODEL=text-embedding-3-small`, `ELASTICSEARCH_URL=http://127.0.0.1:9200`, `DASH0_OTLP_ENDPOINT`, `DASH0_AUTH_TOKEN`, `DASH0_DATASET=default`, `RUST_LOG=info,backend_api=debug`.
- [x] **Create `.gitar/rules.md` (Sonar / Gitar Track — Mandatory)**
  - Define 5 strict repository invariants for Gitar automated PR review and auto-fixing:
    1. All public `axum` handlers in `backend-api/src/routes/*.rs` MUST carry `#[tracing::instrument(skip_all, fields(http.route = ...))]` for Dash0 observability.
    2. All timestamps across models and Elasticsearch payloads MUST use `chrono::DateTime<chrono::Utc>` (RFC3339 UTC). Naive timestamps are forbidden.
    3. Route handlers and services MUST NOT use `.unwrap()` or `.expect()` on fallible I/O, HTTP, or JSON operations; propagate via `?` into `ApiError`.
    4. All session cookies MUST enforce `HttpOnly`, `SameSite::Lax`, and HMAC-SHA256 signature verification.
    5. All talk and room queries MUST preserve `accessibility` and `decibel_level` metadata required by the Merge Forward sensory router.
- [x] **Create `.github/workflows/ci.yml`**
  - Configure GitHub Actions workflow triggered on `pull_request` and `push` to `main`:
    - Job 1 (`rust-check`): Runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --workspace`.
    - Job 2 (`frontend-check`): Runs `npm ci`, `npm run typecheck`, and `npm run build` in `frontend-pwa/`.
- [x] **Create `Caddyfile`**
  - Document reverse proxy for `vector.winslow.plus` -> `127.0.0.1:8080` with `flush_interval -1` (for zero-buffering SSE streams) and `encode zstd gzip`.
- [x] **Create `docker-compose.yml`**
  - Define `elasticsearch` service (`docker.elastic.co/elasticsearch/elasticsearch:8.15.0`) with `discovery.type=single-node`, `xpack.security.enabled=false`, `ES_JAVA_OPTS=-Xms1g -Xmx1g`, port `9200:9200`, and healthcheck on `http://localhost:9200/_cluster/health`.

### 1.2 Kubernetes Manifests for Radar Track (`k8s/`)
- [x] **Create `k8s/namespace.yaml`**
  - Define namespace `vector-prod` with labels `app.kubernetes.io/part-of: vector-kubecon` and `radar.skyhook.io/monitored: "true"`.
- [x] **Create `k8s/elasticsearch-statefulset.yaml`**
  - Define `StatefulSet` and headless `Service` (`elasticsearch:9200`) inside `vector-prod` with readiness probes and memory limits (`2Gi`).
- [x] **Create `k8s/vector-api-deployment.yaml`**
  - Define `Deployment` (`vector-api`, 2 replicas) and `Service` (`NodePort` `30080` -> container port `8080`, mapped to host `18081`) with `/api/health` liveness and readiness probes, env vars from `Secret`/`ConfigMap`, and resource requests/limits (`cpu: 250m/1000m`, `memory: 256Mi/512Mi`).
- [x] **Create `Dockerfile`**
  - Multi-stage build: Stage 1 (`node:22-alpine`) builds `frontend-pwa/dist`; Stage 2 (`rust:1.82-bookworm`) compiles `backend-api` and `crowd-simulator` in `--release`; Stage 3 (`debian:bookworm-slim` with `ca-certificates` and `libssl3`) copies binaries, `seed/`, and `frontend-pwa/dist` into `/app`.

### 1.3 Rust Workspace & Core Telemetry (`backend-api/src/`)
- [x] **Create Root `Cargo.toml` and `backend-api/Cargo.toml` + `crowd-simulator/Cargo.toml`**
  - Configure workspace members `["backend-api", "crowd-simulator"]` and all dependencies (`axum 0.7`, `axum-extra 0.9`, `tokio`, `reqwest 0.12`, `serde`, `serde_json`, `chrono`, `uuid`, `hmac`, `sha2`, `base64`, `opentelemetry 0.26`, `opentelemetry_sdk 0.26`, `opentelemetry-otlp 0.26`, `tracing-opentelemetry 0.27`, `tower-http 0.6`).
- [x] **Create `backend-api/src/config.rs` (~140 LOC)**
  - Implement `AppConfig::from_env()` loading all environment variables with sensible local defaults so `cargo test` and local dev boot cleanly even if optional keys are missing.
- [x] **Create `backend-api/src/error.rs` (~160 LOC)**
  - Implement `ApiError` enum (`BadRequest`, `Unauthorized`, `NotFound`, `ElasticError`, `OpenAiError`, `GitHubError`, `InternalError`) implementing `axum::response::IntoResponse`.
  - Automatically extract the current OpenTelemetry `trace_id` from the active `tracing::Span` and return structured JSON:
    `{ "error": { "code": "...", "message": "...", "trace_id": "...", "timestamp_utc": "..." } }`.
- [x] **Create `backend-api/src/telemetry.rs` (~280 LOC) (Dash0 $4,000 Track Core)**
  - Initialize `opentelemetry_otlp` trace exporter and periodic metric reader pointed at `DASH0_OTLP_ENDPOINT` with `Authorization: Bearer <DASH0_AUTH_TOKEN>` headers (falling back gracefully to stdout structured JSON spans when `DASH0_OTLP_ENDPOINT` is empty during unit tests).
  - Create singleton `TelemetryMetrics` struct holding OpenTelemetry instruments:
    - `room_occupancy_ratio: Gauge<f64>` (`vector.room.occupancy_ratio`)
    - `room_decibels: Gauge<f64>` (`vector.room.decibels`)
    - `surge_reroutes_total: Counter<u64>` (`vector.surge.reroutes_total`)
    - `elastic_query_latency_ms: Histogram<f64>` (`vector.elastic.rrf_latency_ms`)
    - `openai_tokens_total: Counter<u64>` (`vector.gen_ai.tokens_total`)
    - `active_sse_clients: UpDownCounter<i64>` (`vector.sse.active_clients`)
  - Provide helper `current_trace_id() -> String` to attach live Dash0 trace IDs to API responses and SSE events.
- [x] **Create `backend-api/src/state.rs` (~120 LOC)**
  - Define `AppState` (`Arc<InnerState>`) holding `config: AppConfig`, `http: reqwest::Client`, `elastic: ElasticClient`, `openai: OpenAiEngine`, `github: GitHubScraper`, `metrics: TelemetryMetrics`, and `sse_tx: tokio::sync::broadcast::Sender<SseEvent>`.
- [x] **Create minimal `backend-api/src/main.rs` and `crowd-simulator/src/main.rs` stubs**
  - Expose `GET /api/health` returning `{ "status": "ok", "service": "vector-api", "trace_id": "...", "timestamp_utc": "..." }`.

### 1.4 Phase 1 Verification Gate & PR #1
- [x] Run `cargo check --workspace` and `cargo test --workspace` — must exit `0` with zero warnings.
- [x] Start Elasticsearch (`docker compose up -d elasticsearch`) and verify `curl -s http://127.0.0.1:9200` returns cluster info.
- [x] Commit all files on `feat/phase-1-workspace-and-infra`, push to GitHub, open **PR #1**, verify Gitar/CI runs, and merge to `main`.

---

## PHASE 2: Domain Models, Seed Dataset & Elasticsearch 3-Way Hybrid RRF Engine (~1,450 LOC)
**Branch:** `feat/phase-2-elastic-hybrid-rrf`
**Goal:** Build all domain data models, the realistic KubeCon SLC seed dataset (25 talks, 12 rooms, 12 builder/judge profiles), the Elasticsearch 8.15 index manager, the 3-Way Hybrid RRF search engine, the conflict-free DAG schedule solver, and the Merge Forward Sensory Haven room router.

### 2.1 Comprehensive Domain Models (`backend-api/src/models/`)
- [x] **Create `backend-api/src/models/mod.rs`**
- [x] **Create `backend-api/src/models/talk.rs` (~220 LOC)**
  - Define `AccessibilityMeta`: `wheelchair_accessible: bool`, `asl_interpreter: bool`, `live_captioning: bool`, `sensory_load: SensoryLoad` (`Low`, `Medium`, `High`), `quiet_zone_nearby: String`.
  - Define `GeoPoint { lat: f64, lon: f64 }`, `MapPoint { x: f64, y: f64 }`, and `RoomZone` (`MainHall`, `NorthWing`, `SouthWing`, `Mezzanine`).
  - Define `ConferenceTalk`: `talk_id`, `title`, `abstract_text`, `speaker`, `speaker_github`, `company`, `track`, `tags: Vec<String>`, `room_id`, `room_name`, `floor: i32`, `zone: RoomZone`, `coordinates: GeoPoint`, `map_point: MapPoint`, `start_time: DateTime<Utc>`, `end_time: DateTime<Utc>`, `capacity_max: i32`, `capacity_current: i32`, `occupancy_ratio: f64`, `decibel_level: f64`, `accessibility: AccessibilityMeta`, `embedding: Vec<f32>`.
  - Define `ScoredTalk`: wraps `ConferenceTalk` with `rrf_score: f64`, `vector_similarity: f64`, `keyword_score: f64`, `match_reasons: Vec<String>`, `is_rerouted_alternative: bool`.
  - Define `RoomState`: `room_id`, `room_name`, `floor`, `zone`, `coordinates`, `map_point: MapPoint`, `capacity_max`, `capacity_current`, `occupancy_ratio`, `decibel_level`, `active_talk_id: Option<String>`, `active_talk_title: Option<String>`, `next_talk_start: Option<DateTime<Utc>>`, `is_sensory_haven: bool`, `accessibility: AccessibilityMeta`, `updated_at: DateTime<Utc>`.
- [x] **Create `backend-api/src/models/builder.rs` (~200 LOC)**
  - Define `SensoryPreferences`: `sensory_mode_enabled: bool`, `max_crowd_ratio: f64`, `max_decibels: f64`, `require_wheelchair_access: bool`, `require_live_captions: bool`, `prefer_quiet_zones: bool`.
  - Define `RepoSummary`: `name: String`, `description: String`, `language: String`, `stars: u32`, `topics: Vec<String>`, `readme_excerpt: String`.
  - Define `BuilderProfile`: `username: String`, `github_id: u64`, `display_name: String`, `avatar_url: String`, `bio: String`, `company: Option<String>`, `github_url: String`, `linkedin_url: Option<String>`, `email: Option<String>`, `persona_title: String`, `dna_tags: Vec<String>`, `top_languages: Vec<String>`, `hallway_icebreaker: String`, `ideal_talk_query: String`, `top_repos: Vec<RepoSummary>`, `current_room_id: String`, `coordinates: GeoPoint`, `sensory_prefs: SensoryPreferences`, `bookmarked_talk_ids: Vec<String>`, `bumped_from_talk_id: Option<String>`, `pin_code: String`, `embedding: Vec<f32>`, `updated_at: DateTime<Utc>`.
  - Define `MatchedBuilder`: wraps public `BuilderProfile` (excluding raw 1536-float embedding vector to save bandwidth) with `similarity_pct: f64`, `shared_tags: Vec<String>`, `shared_context_reason: String`, `distance_meters: f64`.
- [x] **Create `backend-api/src/models/handshake.rs` (~130 LOC)**
  - Define `QrHandshakeToken`: signed payload `{ username, pin_code, current_room_id, active_talk_id, issued_at_utc, signature }`.
  - Define `HandshakeRecord`: `handshake_id: String`, `initiator_username: String`, `peer_username: String`, `peer_display_name: String`, `peer_avatar_url: String`, `peer_persona_title: String`, `peer_github_url: String`, `peer_linkedin_url: Option<String>`, `met_room_id: String`, `met_room_name: String`, `met_during_talk_title: Option<String>`, `shared_dna_tags: Vec<String>`, `ai_context_summary: String`, `draft_linkedin_dm: String`, `draft_email_subject: String`, `draft_email_body: String`, `created_at: DateTime<Utc>`.
- [x] **Create `backend-api/src/models/sse.rs` (~120 LOC)**
  - Define `SseEvent` tagged enum (`#[serde(tag = "event_type", content = "data")]`):
    - `RoomTelemetryUpdate { room: RoomState, trace_id: String, timestamp_utc: DateTime<Utc> }`
    - `CrowdSurgeAlert { jammed_room_id: String, jammed_room_name: String, jammed_talk_id: String, jammed_talk_title: String, occupancy_current: i32, occupancy_max: i32, decibel_level: f64, recommended_alternative: Option<ScoredTalk>, bumped_hallway_peers: Vec<MatchedBuilder>, trace_id: String, timestamp_utc: DateTime<Utc> }`
    - `DnaSyncProgress { username: String, step: String, progress_pct: u8, detail: String, timestamp_utc: DateTime<Utc> }`
    - `HandshakeCompleted { handshake: HandshakeRecord, trace_id: String, timestamp_utc: DateTime<Utc> }`
    - `SensoryHavenUpdate { recommended_haven: RoomState, reason: String, timestamp_utc: DateTime<Utc> }`

### 2.2 Rich Seed Datasets (`seed/`)
- [x] **Create `seed/kubecon_rooms.json` (~150 LOC)**
  - Define 12 authentic Salt Palace Convention Center rooms across Floors 1 & 2 (`ballroom-b`, `room-250a`, `room-251c`, `room-155b`, `room-204b-quiet`, `room-355e`, `hall-4-keynote`, `lounge-sensory-north`, etc.) with SVG map `(x, y)` coordinates, `GeoPoint`, baseline capacities, baseline decibels (`36.0` to `78.0` dB), and accessibility flags. Ensure at least 3 rooms have scheduled gaps so the Sensory Haven router finds real empty conference rooms!
- [x] **Create `seed/kubecon_talks.json` (~350 LOC)**
  - Define 25 realistic KubeCon NA Salt Lake City sessions across 5 time slots (11:00 AM, 1:00 PM, 2:00 PM, 3:15 PM, 4:30 PM) covering Rust in Cloud-Native, eBPF & Cilium, Kubernetes Operators, OpenTelemetry at Scale, AI/LLM Inference on K8s, WebAssembly, Platform Engineering, and Security.
  - Ensure the **2:00 PM slot** has:
    - Talk A in `ballroom-b`: *"Zero-Overhead Service Mesh with eBPF and Rust"* (`capacity_max: 150, capacity_current: 142` — ready to surge to `150/150`).
    - Talk B in `room-250a`: *"Building High-Speed Kubernetes Operators & Multiplexers in Rust"* (`capacity_max: 120, capacity_current: 64` — open seats, high semantic overlap for auto-rerouting).
    - Talk C in `room-251c`: *"Deep OpenTelemetry Tracing & Autonomous AI SRE Loops on K8s"* (`capacity_max: 100, capacity_current: 58`).
    - Room `room-204b-quiet`: **0 talks scheduled during the 2:00 PM slot** (`capacity_current: 6, capacity_max: 80, decibel_level: 37.5` — the verified empty conference room for Merge Forward Sensory Routing!).
- [x] **Create `seed/judge_profiles.json` (~200 LOC)**
  - Pre-seed 12 rich builder profiles including hackathon judges and notable cloud-native engineers (`kelseyhightower`, `saiyampathak`, `estesp`, `wesnovack`, `joesandoval`, `simonschrottner`, `tomas-cline`, `mayak8s`, `ebpf-alex`, `rust-sre-dan`, `zircon-dev`) with realistic `dna_tags`, `top_languages`, `hallway_icebreaker`, and deterministic 1536-dim normalized fallback embeddings.

### 2.3 Elasticsearch Client, 3-Way Hybrid RRF & Schedule Solver (`backend-api/src/services/`)
- [x] **Create `backend-api/src/services/elastic_client.rs` (~320 LOC)**
  - Implement raw REST JSON DSL client over `reqwest::Client`.
  - Implement `ensure_indices_and_seed(&self)`: creates `conference_talks`, `builders`, `rooms_state`, and `handshakes` indices with `dense_vector` (`dims: 1536, index: true, similarity: "cosine"`) mappings and bulk-indexes `seed/*.json` if the indices are empty.
  - Implement deterministic fallback 1536-dim semantic hash embedding generator (`generate_deterministic_embedding(text: &str) -> Vec<f32>`) used at initial seed boot so all 25 talks and 12 profiles have valid unit-normalized cosine vectors within 50ms of startup, while `openai_engine.rs` generates live `text-embedding-3-small` vectors during user syncs.
- [x] **Create `backend-api/src/services/hybrid_search.rs` (~300 LOC) (Elasticsearch $4,000 Track Core)**
  - Implement `search_talks_hybrid_rrf(&self, profile: &BuilderProfile, slot_filter: Option<DateTime<Utc>>, exclude_full_rooms: bool) -> Result<(Vec<ScoredTalk>, serde_json::Value), ApiError>`:
    - Executes an Elasticsearch compound query combining:
      1. `knn` vector search on `embedding` (`k: 20, num_candidates: 50, boost: 0.65`).
      2. `bool.should` BM25 `multi_match` across `title^3`, `tags^4`, `abstract_text^1.5`, and `track^2` using `profile.dna_tags` and `profile.top_languages` (`boost: 0.35`).
      3. `bool.filter` enforcing `occupancy_ratio < 0.98` (when `exclude_full_rooms == true`) and accessibility constraints when `profile.sensory_prefs.sensory_mode_enabled == true`.
    - Computes Reciprocal Rank Fusion (RRF) + cosine similarity % and returns **both** the ranked `Vec<ScoredTalk>` AND the exact Elasticsearch JSON query DSL + execution time (`took` ms) so the frontend Judge Control Dock can display the live Elasticsearch query!
  - Implement `match_hallway_builders(&self, profile: &BuilderProfile, room_filter: Option<&str>, bumped_talk_id: Option<&str>) -> Result<Vec<MatchedBuilder>, ApiError>`:
    - Queries the `builders` index via `knn` vector similarity (`embedding`), excluding `profile.username`, boosting peers in the same room or who were also bumped from `bumped_talk_id`.
- [x] **Create `backend-api/src/services/schedule_solver.rs` (~220 LOC)**
  - Implement `build_conflict_free_schedule(scored_talks: &[ScoredTalk], bookmarked_ids: &[String], sensory_mode: bool) -> CustomSchedule`:
    - Groups talks by time slot, selects the optimal conflict-free path maximizing RRF score, injects 15-minute Sensory Decompression Buffers in verified quiet rooms if `sensory_mode == true`, and identifies immediate runner-up backups for every slot in case of a crowd surge.
- [x] **Create `backend-api/src/services/sensory_router.rs` (~160 LOC) (Merge Forward iPad Track Core)**
  - Implement `find_optimal_sensory_haven(&self, current_zone: &RoomZone) -> Result<Vec<RoomState>, ApiError>`:
    - Queries `rooms_state` in Elasticsearch for rooms where `active_talk_id` is `null` (unused conference room right now) OR `is_sensory_haven == true`, filtered by `occupancy_ratio < 0.20` and `decibel_level < 45.0`, sorted by lowest decibel level and proximity to `current_zone`.

### 2.4 Phase 2 Verification Gate & PR #2
- [x] Write integration test `backend-api/tests/elastic_rrf_test.rs` that boots `ElasticClient`, seeds the 25 talks and 12 builders, executes `search_talks_hybrid_rrf`, `match_hallway_builders`, and `find_optimal_sensory_haven`, and asserts non-empty ranked results and valid RRF scores.
- [x] Run `cargo test --workspace` — must pass 100%.
- [x] Commit on `feat/phase-2-elastic-hybrid-rrf`, push to GitHub, open **PR #2**, verify Gitar/CI checks, and merge to `main`.

---

## PHASE 3: GitHub OAuth, Signed Cookies, OpenAI GPT-6.1 Sol Pipeline & API Routes (~1,400 LOC)
**Branch:** `feat/phase-3-auth-openai-and-routes`
**Goal:** Implement HMAC-SHA256 signed `HttpOnly` session cookies, real GitHub OAuth2 + 1-click Judge Impersonation, concurrent GitHub repository scraping, OpenAI GPT-6.1 Sol Structured Outputs + Embeddings, and all REST + SSE endpoints instrumented with Dash0 OpenTelemetry spans.

### 3.1 Security, Signed Cookies & GitHub OAuth (`backend-api/src/auth/`)
- [x] **Create `backend-api/src/auth/cookies.rs` (~180 LOC)**
  - Implement `SessionToken`: `{ username: String, github_id: u64, iat: i64, exp: i64 }`.
  - Implement `sign_session_cookie(&SessionToken, secret: &[u8]) -> Cookie<'static>` producing `vector_sid=<base64url_payload>.<base64url_hmac_sha256>` with `HttpOnly`, `SameSite::Lax`, `Path=/`, `Max-Age=86400`, and `Secure` (enabled automatically when `PUBLIC_BASE_URL` starts with `https://`).
  - Implement `verify_session_cookie(jar: &CookieJar, secret: &[u8]) -> Option<SessionToken>` using constant-time HMAC comparison.
- [x] **Create `backend-api/src/auth/middleware.rs` (~100 LOC)**
  - Implement `OptionalAuthUser` and `RequireAuthUser` Axum `FromRequestParts` extractors that verify `vector_sid` (or fallback header `X-Vector-User` for CLI/testing) and attach `enduser.id` to the active OpenTelemetry span.
- [x] **Create `backend-api/src/auth/github_oauth.rs` (~190 LOC)**
  - Implement `build_github_authorize_url(&AppConfig) -> (String, Cookie<'static>)` with CSRF `state` cookie `vector_oauth_state`.
  - Implement `exchange_code_for_token(&AppConfig, code: &str) -> Result<String, ApiError>`.

### 3.2 Live GitHub Scraper & OpenAI GPT-6.1 Sol Engine (`backend-api/src/services/`)
- [x] **Create `backend-api/src/services/github_scraper.rs` (~220 LOC)**
  - Implement `scrape_github_user(&self, username: &str, oauth_token: Option<&str>) -> Result<ScrapedGitHubData, ApiError>`:
    - Uses `tokio::join!` to concurrently fetch `/users/{username}` and `/users/{username}/repos?sort=pushed&per_page=8`.
    - Fetches the first 1,200 characters of `README.md` for the top 2 starred repositories.
    - Includes automatic fallback to `seed/judge_profiles.json` if GitHub unauthenticated rate limits (`403/429`) are hit on conference Wi-Fi!
- [x] **Create `backend-api/src/services/openai_engine.rs` (~280 LOC)**
  - Implement `extract_builder_dna(&self, scraped: &ScrapedGitHubData) -> Result<ExtractedDna, ApiError>`:
    - Calls `https://api.openai.com/v1/chat/completions` using `config.openai_model` (`gpt-6.1-sol`, with automatic fallback to `gpt-4o-mini` if model alias differs) and Structured Outputs (`response_format: { type: "json_schema" }`) to extract `persona_title`, `dna_tags` (5–7 sharp technical tags), `hallway_icebreaker` (referencing their real repos!), and `ideal_talk_query`.
    - Records `gen_ai.request.model` and token usage on the active Dash0 OpenTelemetry span and increments `metrics.openai_tokens_total`.
  - Implement `generate_embedding(&self, text: &str) -> Result<Vec<f32>, ApiError>`:
    - Calls `https://api.openai.com/v1/embeddings` (`text-embedding-3-small`, `dimensions: 1536`).
  - Implement `draft_handshake_followup(&self, initiator: &BuilderProfile, peer: &BuilderProfile, room_name: &str, talk_title: Option<&str>) -> Result<DraftedFollowUp, ApiError>`:
    - Uses `gpt-6.1-sol` Structured Outputs to generate a hyper-specific `ai_context_summary`, `draft_linkedin_dm` (< 280 chars), `draft_email_subject`, and `draft_email_body` referencing both builders' actual GitHub projects and the session they met outside of.

### 3.3 Instrumented Axum Route Handlers (`backend-api/src/routes/`)
*Every route handler MUST include `#[tracing::instrument(skip_all, ...)]` and `chrono::Utc::now()` per `.gitar/rules.md`!*
- [x] **Create `backend-api/src/routes/auth_routes.rs` (~200 LOC)**
  - `GET /api/auth/github`: Redirects to GitHub OAuth authorize URL with `vector_oauth_state` cookie (or if `GITHUB_CLIENT_ID` is not configured, redirects smoothly to `/api/auth/impersonate-default`).
  - `GET /api/auth/github/callback`: Exchanges OAuth code, triggers live GitHub + OpenAI DNA sync, indexes builder into Elasticsearch, sets signed `vector_sid` cookie, and redirects to `/`.
  - `POST /api/auth/impersonate`: Accepts `{ "username": "kelseyhightower" }`, runs the live GitHub scraper + OpenAI DNA extraction + Elasticsearch indexing pipeline (emitting real-time `DnaSyncProgress` SSE events at 20%, 50%, 80%, 100%), sets the `vector_sid` cookie, and returns the full `BuilderProfile`.
  - `POST /api/auth/logout`: Clears `vector_sid` cookie.
- [x] **Create `backend-api/src/routes/profile_routes.rs` (~160 LOC)**
  - `GET /api/me`: Returns current authenticated `BuilderProfile` from cookie (or defaults to `zircon-dev` demo profile if unauthenticated so the landing experience is never broken) + active `trace_id`.
  - `POST /api/sync-github`: Accepts `{ "username": "...", "linkedin_url": "..." }`, runs full live scrape + OpenAI + Elasticsearch re-indexing, sets `vector_sid` cookie, and returns updated profile + schedule.
  - `PATCH /api/sensory`: Toggles `sensory_mode_enabled` and accessibility filters in Elasticsearch, broadcasts `SensoryHavenUpdate` via SSE, and returns updated schedule + quiet room haven.
- [x] **Create `backend-api/src/routes/schedule_routes.rs` & `search_routes.rs` (~220 LOC)**
  - `GET /api/state`: Single consolidated hydration endpoint returning `{ profile, schedule, scored_talks, rooms, sensory_havens, hallway_matches, handshakes, last_elastic_query_dsl, trace_id }`.
  - `POST /api/search/talks`: Executes live 3-Way Hybrid RRF search with custom query text or tag overrides and returns ranked talks + raw Elasticsearch DSL JSON.
  - `POST /api/schedule/bookmark`: Toggles a talk bookmark in Elasticsearch and recalculates conflict-free DAG schedule.
- [x] **Create `backend-api/src/routes/handshake_routes.rs` (~180 LOC)**
  - `GET /api/handshake/badge`: Returns signed `QrHandshakeToken` + 4-digit PIN for the current user.
  - `POST /api/handshake/connect`: Accepts `{ "peer_username": "...", "pin_code": "...", "room_id": "..." }`, calls `openai_engine.draft_handshake_followup`, indexes `HandshakeRecord` into Elasticsearch `handshakes`, broadcasts `HandshakeCompleted` over SSE, and returns the record.
- [x] **Create `backend-api/src/routes/sse_routes.rs` (~110 LOC)**
  - `GET /api/events/stream`: Subscribes to `state.sse_tx`, increments/decrements `metrics.active_sse_clients`, and streams JSON SSE events with 15-second keep-alive heartbeats.

### 3.4 Phase 3 Verification Gate & PR #3
- [x] Write integration test `backend-api/tests/auth_and_routes_test.rs` verifying HMAC cookie signing/verification, `GET /api/state`, `POST /api/auth/impersonate` (for `kelseyhightower`), `PATCH /api/sensory`, and `POST /api/handshake/connect`.
- [x] Run `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace` — must pass 100%.
- [x] Commit on `feat/phase-3-auth-openai-and-routes`, push to GitHub, open **PR #3**, verify Gitar review, and merge to `main`.

---

## PHASE 4: Crowd Simulator CLI, Surge Engine & The "Sponsor Rubric Trap" (~750 LOC)
**Branch:** `feat/phase-4-crowd-surge-and-sre-trap`
**Goal:** Build `/api/rooms/*` and `/api/surge`, build the standalone `crowd-simulator` CLI load generator, and execute the **Mandatory Two-Step Sponsor Rubric Trap** so Sonar Gitar, Dash0 Agent0, and Radar AI SRE all catch and verify a real bug fix in our Git commit history!

### 4.1 Room Telemetry & Crowd Surge Engine (`backend-api/src/routes/room_routes.rs`)
- [x] **Create `backend-api/src/routes/room_routes.rs` (~260 LOC)**
  - `GET /api/rooms`: Returns all 12 Salt Palace rooms with live occupancy ratios, decibel readings, and active talk metadata.
  - `POST /api/rooms/checkin`: Accepts `{ "room_id": "...", "delta": 5, "decibel_reading": 74.2 }`, updates Elasticsearch `rooms_state` and `conference_talks`, records `metrics.room_occupancy_ratio` and `metrics.room_decibels` to Dash0, and broadcasts `RoomTelemetryUpdate` via SSE.
  - `POST /api/surge`: Accepts `{ "room_id": "ballroom-b" }` (defaulting to `ballroom-b`):
    1. Spikes `ballroom-b` occupancy to `150/150 (100% FULL)` and `decibel_level` to `84.5 dB` in Elasticsearch.
    2. Emits a structured OpenTelemetry warning event & span (`tracing::warn!(room.id = %room_id, room.occupancy_ratio = 1.0, "CROWD_SURGE_THRESHOLD_EXCEEDED: triggering 3-way RRF auto-reroute")`) and increments `metrics.surge_reroutes_total`.
    3. Executes `search_talks_hybrid_rrf` with `exclude_full_rooms = true` for the current user's 2:00 PM slot to find the #1 open alternative talk (`room-250a`).
    4. Queries `match_hallway_builders` to find 2 high-similarity peers who were also just bumped outside `ballroom-b` (`mayak8s`, `kelseyhightower`).
    5. Broadcasts `CrowdSurgeAlert` over SSE and returns the full surge resolution payload + Dash0 `trace_id`.
  - `POST /api/surge/reset`: Resets `ballroom-b` back to `142/150 (94.6%)` and `68.0 dB` so the live demo can be re-triggered infinitely on stage!

### 4.2 Standalone Crowd Simulator CLI (`crowd-simulator/src/main.rs`)
- [x] **Implement `crowd-simulator/src/main.rs` (~240 LOC)**
  - CLI flags: `--target http://127.0.0.1:8080`, `--mode <wave|surge|reset>`, `--concurrency 25`.
  - In `wave` mode: Spawns concurrent tokio tasks simulating attendees checking into rooms, querying `/api/search/talks`, and emitting realistic telemetry to Dash0 and Radar.
  - In `surge` mode: Fires a burst of check-ins to `ballroom-b` culminating in `POST /api/surge`, printing the Dash0 `trace_id` and rerouted talk to the terminal.

### 4.3 THE MANDATORY TWO-COMMIT SPONSOR RUBRIC TRAP (Do Not Skip!)
*Why this exists: The judging rubrics for Sonar Gitar ($4k), Dash0 ($4k), and Radar ($4k) explicitly award top points for showing a real issue caught and fixed during development.*
- [ ] **Step 4.3A (Commit 1 on `feat/phase-4-crowd-surge-and-sre-trap` — The Deliberate Bottleneck):**
  - Add a diagnostic endpoint `POST /api/rooms/audit-unbatched` that intentionally performs an unbatched N+1 sequential Elasticsearch HTTP loop (15 sequential requests inside a loop) AND omits `#[tracing::instrument]`, then push Commit 1 to `feat/phase-4-crowd-surge-and-sre-trap` and open **PR #4**.
- [ ] **Step 4.3B (Commit 2 on `feat/phase-4-crowd-surge-and-sre-trap` — The Verified Fix):**
  - Immediately replace the N+1 sequential loop in `POST /api/rooms/audit-unbatched` with a single batched Elasticsearch `_mget` / single query, add `#[tracing::instrument(skip_all)]` and UTC timestamps per `.gitar/rules.md`, and document the before/after latency (`~185ms -> ~8ms`) in `docs/SRE_INCIDENT_REPORT.md` so the user can show the exact commit diff and incident report to the Dash0, Radar, and Sonar judges!

### 4.4 Phase 4 Verification Gate & PR #4
- [ ] Run `cargo test --workspace` and test `POST /api/surge` followed by `POST /api/surge/reset`.
- [ ] Push Commit 2 to **PR #4**, verify CI passes cleanly, and merge to `main`.

---

## PHASE 5: Sleek Vue 3 + TypeScript + Tailwind CSS Spatial Mobile PWA & Desktop Judge Dock (~2,150 LOC)
**Branch:** `feat/phase-5-vue-spatial-pwa-and-judge-dock`
**Goal:** Build the ultra-polished obsidian dark-mode Vue 3 (`<script setup lang="ts">`) + Tailwind CSS + Pinia frontend inside `frontend-pwa/`, featuring the Unified "Uber / Flighty" Spatial Radar UX inside the 430px Mobile Shell on the left + Live Judge & Chaos Control Dock on the right on desktop screens.

### 5.1 Frontend Scaffolding & Typed API / SSE Store Layer (`frontend-pwa/`)
- [ ] **Initialize `frontend-pwa/` (`package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`)**
  - Dependencies: `vue@^3.5`, `pinia@^2.2`, `lucide-vue-next`, `qrcode@^1.5` (plus `@types/qrcode` for instant client-side SVG/Canvas QR generation), `tailwindcss@^3.4`, `postcss`, `autoprefixer`.
  - Configure `vite.config.ts` with dev proxy `/api -> http://127.0.0.1:8080` and build output `outDir: 'dist'`.
  - Load `Inter` and `JetBrains Mono` Google Fonts in `index.html`, set viewport theme-color `#07090E`, and title `VECTOR // Zero-Config KubeCon OS`.
- [ ] **Create `frontend-pwa/src/styles/main.css` (~140 LOC)**
  - Define obsidian dark-mode theme variables, subtle radial grid backdrop, glassmorphic cards (`backdrop-blur-xl bg-[#0F1420]/90 border border-white/10`), smooth Vue `<Transition>` and `<TransitionGroup>` FLIP animations (`v-move`), and pulsing telemetry indicator keyframes.
- [ ] **Create `frontend-pwa/src/types/vector.ts` (~180 LOC)**
  - Exact 1:1 TypeScript interfaces matching all Rust structs in `backend-api/src/models/*.rs`.
- [ ] **Create `frontend-pwa/src/api/client.ts` (~160 LOC)**
  - Typed `apiFetch<T>(path, options)` wrapper with `credentials: 'include'`, automatic latency timing (`lastRequestMs`), and active `trace_id` capture.
- [ ] **Create `frontend-pwa/src/stores/vectorStore.ts` (~320 LOC)**
  - Consolidated Pinia store managing:
    - `profile`, `schedule`, `scoredTalks`, `rooms`, `sensoryHavens`, `hallwayMatches`, `handshakes`.
    - `selectedSlotIndex`, `isMapExpanded`, `isVaultOpen`, `isQrModalOpen`.
    - `dnaSyncSteps`: real-time progress array populated during GitHub OAuth / Judge Impersonation.
    - `activeSurgeAlert`: populated immediately when `CrowdSurgeAlert` arrives over SSE or `triggerCrowdSurge()` is clicked.
    - `lastElasticDsl`, `lastTraceId`, `sseConnected`, `sseEventLog` (ring buffer of last 25 real-time SSE events for the Judge Control Dock).
    - `connectSseStream()`: opens `new EventSource('/api/events/stream')` with auto-reconnect and reactive state mutation on every event variant.

### 5.2 Mobile Shell, Onboarding Terminal & Top Telemetry Pill (`src/components/`)
- [ ] **Create `src/components/layout/TopTelemetryPill.vue` (~130 LOC)**
  - Floating glass header inside the mobile viewport displaying:
    - Live status dot + active room status (e.g., `🟢 Ballroom B · 94%` or `🔴 SURGE REROUTED · Rm 250A`).
    - 1-Tap **[🧠 Quiet]** toggle button (`PATCH /api/sensory`) that shifts accent glow to calming teal (`#14B8A6`).
    - 1-Tap **[🏁 QR]** button opening `QrBadgeModal.vue`.
- [ ] **Create `src/components/onboarding/GithubLoginHero.vue` & `DnaExtractionTerminal.vue` (~240 LOC)**
  - Compact collapsible switcher + modal allowing instant switching between:
    - **[Sign in with GitHub OAuth]** (`/api/auth/github`),
    - **1-Click Judge Presets** (`@kelseyhightower`, `@saiyampathak`, `@estesp`, `@wesnovack`, `@zircon-dev`), or
    - **Any Custom GitHub Handle** input box + optional LinkedIn URL.
  - When triggered, displays a sleek animated terminal streaming the real SSE steps (`Fetching GitHub repos...` -> `GPT-6.1 Sol extracting Builder DNA...` -> `Embedding 1536-dim vector...` -> `Executing Elasticsearch 3-Way RRF...`) and reveals the user's glowing DNA tags & custom Hallway Icebreaker.

### 5.3 The Unified "Uber / Flighty" Spatial Radar UX (`src/components/`)
*CRITICAL UX DIRECTIVE: Do NOT hide the Map, Schedule, and Hallway Matches behind separate isolated tabs. Render a unified vertical Spatial Live Radar inside `MobileShell.vue` so the Map, Active Talk Card, and Nearby Hallway Peers react simultaneously in a single frame!*
- [ ] **Create `src/components/map/SaltPalaceSpatialMap.vue` (~300 LOC) (Top 42% of Mobile Viewport — Collapsible/Expandable)**
  - Interactive dark-mode architectural SVG floorplan of the Salt Palace Convention Center rooms always visible at the top of the mobile screen (with a `[⤢ Expand]` button).
  - Rooms color dynamically by live `occupancy_ratio` (Emerald `<70%`, Amber `70–94%`, Crimson `>=95% FULL`, Calming Teal pulsing border for verified **Empty Sensory Haven Rooms**).
  - **Animated SVG Reroute Path (`<path stroke-dasharray="6,6">`):**
    - When `Ballroom B` surges to 100% FULL, automatically draws an animated glowing emerald vector path from `Ballroom B` to the rerouted open room (`Room 250A`).
    - When **Quiet / Sensory Mode** is toggled on, automatically draws a calming teal vector path from the user's current room to the quietest empty room (`Room 204B · 37.5 dB`).
- [ ] **Create `src/components/feed/UnifiedLiveSheet.vue`, `SurgeRerouteAlert.vue` & `SensoryHavenBanner.vue` (~380 LOC) (Bottom 58% of Mobile Viewport)**
  - **Compact Builder DNA Bar:** Displays user avatar, `persona_title`, glowing `dna_tags`, and expandable `hallway_icebreaker` at the top of the sheet.
  - **Horizontal Time-Slot Scrubber:** Pills for `11:00 AM` · `1:00 PM` · `● 2:00 PM NOW` · `3:15 PM` · `4:30 PM`. Tapping any slot highlights that slot's recommended room on the map above and updates the card below.
  - **Merge Forward Sensory Haven Card (`SensoryHavenBanner.vue`):** When Quiet Mode is active, pins a calming teal card showing the #1 verified empty conference room (`Room 204B · 0 Talks Scheduled Until 3:30 PM · 7% Occupancy · 37.5 dB · Wheelchair & Low-Sensory Certified`).
  - **Active Session & Auto-Reshuffle Card:** Shows the active slot's top RRF talk with live capacity bar & decibel badge. When a surge hits, smoothly morphs via Vue `<Transition>` to show the crimson `Ballroom B (150/150 FULL)` banner transitioning into the `#2 RRF Match (Room 250A · 96.8% Match · 56 Seats Open)`.
- [ ] **Create `src/components/hallway/HallwayProximityStrip.vue` & `QrBadgeModal.vue` (~300 LOC) (Pinned Directly Below Active Card)**
  - Horizontal / compact stack of the top 3 vector-matched attendees in the user's current zone (or who were also bumped from `Ballroom B` during a surge), showing avatar, `98.4% DNA Match`, shared tags, and a 1-tap **[⚡ Handshake]** button.
  - Tapping **[⚡ Handshake]** or the top **[QR Badge]** pill opens `QrBadgeModal.vue` (full-screen scannable QR code + 4-digit PIN `#4829` + live OpenAI GPT-6.1 Sol LinkedIn/Email follow-up generator).
- [ ] **Create `src/components/hallway/ConnectionVaultDrawer.vue` (~200 LOC) (Slide-Up Bottom Drawer)**
  - Persistent bottom bar (`🤝 Saved Connections Vault (N)`) that slides up a full drawer showing all captured hallway handshakes, meeting room/talk context, and 1-click **[Copy LinkedIn DM]** / **[Open Mail Client ↗]** buttons.

### 5.4 The Desktop Judge & Chaos Control Dock (`src/components/layout/JudgeControlDock.vue`)
- [ ] **Create `src/components/layout/JudgeControlDock.vue` (~300 LOC)**
  - Rendered on the right half of desktop screens (`>= 1024px`) next to the `430px` mobile device frame so judges watching the projector see both the mobile UX AND the real cloud-native backend telemetry simultaneously:
    1. **1-Click Chaos & Demo Controls:**
       - **[⚡ Trigger Ballroom B Crowd Surge (`POST /api/surge`)]** & **[↺ Reset Surge]**
       - **[🧠 Toggle Merge Forward Sensory-Safe Mode (`PATCH /api/sensory`)]**
       - **[🎤 Sample Live Room Noise (`POST /api/rooms/checkin`)]**
       - **Judge GitHub Quick-Sync Pills:** `@kelseyhightower`, `@saiyampathak`, `@estesp`, `@wesnovack`, `@zircon-dev`.
    2. **Live Sponsor Track Proof Panel (4 Tabs):**
       - **Dash0 OpenTelemetry Live Stream:** Displays active `trace_id`, request latency, live `vector.room.occupancy_ratio` gauges, and SSE event feed.
       - **Elasticsearch 8.15 RRF Query Inspector:** Displays the live JSON DSL (`knn` + `multi_match` + capacity/accessibility filters) sent to Elasticsearch on the last request.
       - **Radar K8s + SRE Status:** Shows pod health, replica status, and the `SRE_INCIDENT_REPORT.md` N+1 batch fix summary.
       - **Live Phone QR Link:** Renders a scannable QR code pointing straight to `https://vector.winslow.plus` so any judge can scan the projector screen and test the live app on their own phone!

### 5.5 Phase 5 Verification Gate & PR #5
- [ ] Run `npm run build` inside `frontend-pwa/` — must compile cleanly with zero TypeScript or Vite errors and generate `frontend-pwa/dist/index.html`.
- [ ] Commit on `feat/phase-5-vue-spatial-pwa-and-judge-dock`, push to GitHub, open **PR #5**, verify Gitar/CI passes, and merge to `main`.

---

## PHASE 6: Monolith SPA Serving, End-to-End Integration Tests & Live Production Polish (~400 LOC)
**Branch:** `feat/phase-6-production-deploy-and-e2e`
**Goal:** Wire `backend-api` to serve `frontend-pwa/dist` with SPA fallback on port `8080`, write a comprehensive automated end-to-end test suite exercising every single demo flow, and verify `http://127.0.0.1:8080` (and `https://vector.winslow.plus`) is 100% ready for stage presentation.

### 6.1 Static SPA Serving & Production Wiring (`backend-api/src/main.rs`)
- [ ] **Update `backend-api/src/main.rs` (~180 LOC)**
  - Mount all `/api/*` routes with `tower_http::trace::TraceLayer` and compression.
  - Serve static assets from `../frontend-pwa/dist` (or `./frontend-pwa/dist` / `STATIC_DIR` env var) via `ServeDir::new(static_dir).not_found_service(ServeFile::new(index_html))` so client-side routing and `https://vector.winslow.plus` work seamlessly from a single binary.
  - On startup, automatically run `elastic.ensure_indices_and_seed()` and initialize room occupancy gauges in Dash0 OpenTelemetry.

### 6.2 End-to-End Automated Demo Verification Suite (`backend-api/tests/e2e_demo_flow_test.rs`)
- [ ] **Create `backend-api/tests/e2e_demo_flow_test.rs` (~220 LOC)**
  - Automated test spinning up the full Axum router and verifying the exact 3-minute hackathon pitch sequence:
    1. `GET /api/health` -> `200 OK` with valid `trace_id`.
    2. `GET /api/state` -> returns seeded KubeCon talks, 12 rooms, sensory havens, and builder profile.
    3. `POST /api/auth/impersonate` with `{ "username": "kelseyhightower" }` -> verifies `Set-Cookie: vector_sid=...; HttpOnly; SameSite=Lax` is returned and profile contains extracted DNA tags.
    4. `POST /api/surge` with `{ "room_id": "ballroom-b" }` -> verifies `ballroom-b` hits `1.0` occupancy ratio, returns rerouted talk in `room-250a`, and returns bumped hallway peers.
    5. `PATCH /api/sensory` with `{ "sensory_mode_enabled": true }` -> verifies `room-204b-quiet` is returned as the top unused quiet conference room haven (`decibel_level < 45.0`).
    6. `POST /api/handshake/connect` between `kelseyhightower` and `zircon-dev` -> verifies AI-drafted LinkedIn DM and Email follow-up are persisted in Elasticsearch `handshakes`.
    7. `POST /api/surge/reset` -> restores `ballroom-b` to pre-surge state cleanly.

### 6.3 Final Verification Gate & PR #6
- [ ] Run `cargo test --workspace` — all unit and E2E tests must pass 100%.
- [ ] Start `backend-api` on port `8080` and verify `curl -I http://127.0.0.1:8080` serves the Vue SPA `index.html` and `curl -s http://127.0.0.1:8080/api/health` returns `200 OK`.
- [ ] Commit final state (with all `[x]` boxes checked in `IMPLEMENTATION.md`) on `feat/phase-6-production-deploy-and-e2e`, push to GitHub, open **PR #6**, let Sonar Gitar complete its final review, and merge to `main`.
