# Room audit performance exercise

## Scope and provenance

This was an intentionally introduced diagnostic bottleneck required by Phase 4 of
`IMPLEMENTATION.md`, not an independently detected production incident. The
baseline is commit `04a7d05` on PR #4. It deliberately omitted handler tracing and
performed 15 sequential Elasticsearch document reads. The follow-up fix adds
route instrumentation and uses one `_mget` request.

The legacy path, `POST /api/rooms/audit-unbatched`, is retained for comparison.
Both versions return the same 15 ordered room samples, including three repeated
rooms, preserving accessibility, decibel readings, and UTC timestamps. The fixed
version checks per-document failures rather than accepting partial results.

## Local evidence

Measurements use the real local Elasticsearch 8.15.0 service and the Axum router
on a WSL VM capped at 3 GB RAM with 2 GB swap. Seven calls are measured with
`Instant` inside the handler, including HTTP and deserialization. These are local
wall-clock timings, not exported Dash0 span durations or production SLOs.

| Audit | Elasticsearch requests | Median latency |
| --- | ---: | ---: |
| Sequential baseline | 15 | 173.843 ms |
| Batched fix | 1 | 16.537 ms |

The measured median improved by approximately 10.5× (90.5% less latency).
Batched samples in milliseconds: 11.017, 13.331, 14.726, 16.537, 17.619,
17.806, 587.369.

Baseline samples in milliseconds: 89.620, 126.536, 140.861, 173.843, 188.592,
281.232, 988.097. Shared VM load and cold-cache effects produce substantial
variation; these samples do not establish a production percentile guarantee.

Reproduce the comparison using the baseline and fix commits:

```bash
CARGO_BUILD_JOBS=1 cargo test -p backend-api --test room_routes_test \
  room_audit_retains_all_samples_and_reports_latency -- --nocapture
```

The regression test validates the sample count, ordering, metadata, and one
request reported by the fixed implementation. The baseline test omits the
one-request assertion. Other integration coverage exercises concurrent arrivals,
capacity bounds, invalid requests, live talk metadata, surge SSE, alternatives,
and reset. The simulator was also exercised against a temporary API on port
18088 in `wave`, `surge`, and `reset` modes with concurrency 25; all succeeded,
and the surge selected Room 250A. Test-created room telemetry was restored.

## Observability and limits

The fixed handler carries `#[tracing::instrument(skip_all, fields(http.route =
"/api/rooms/audit-unbatched"))]`, returns a trace ID and UTC timestamp, and exposes
elapsed time plus request count. Normal room updates record occupancy and decibel
gauges, while surges increment the reroute counter and broadcast contextual SSE.

No Dash0 screenshots, verified Dash0 links, Radar incident detections, or Gitar
review findings were available when this report was written. These are not
claimed as evidence. PR #4 and its CI results provide the reviewable code history.

Room occupancy increments are atomic Elasticsearch scripts. An instance-local
mutex orders room-to-talk propagation and SSE on that instance; multiple API
replicas do not share that mutex, and updates spanning room and talk documents
are not transactional. Production multi-writer consistency remains a limitation.
