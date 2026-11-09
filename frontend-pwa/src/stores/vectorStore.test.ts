import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useVectorStore } from "./vectorStore";
import type {
  BuilderProfile,
  HandshakeRecord,
  RoomState,
  SseEvent,
  StateResponse,
} from "../types/vector";
const room = {
  room_id: "ballroom-b",
  active_talk_id: "talk-a",
  occupancy_ratio: 0.5,
  capacity_current: 75,
  decibel_level: 60,
} as RoomState;
const profile = {
  username: "alice",
  sensory_prefs: { sensory_mode_enabled: false },
} as BuilderProfile;
const record = (id: string, user = "alice") =>
  ({ handshake_id: id, initiator_username: user }) as HandshakeRecord;
const state = () =>
  ({
    profile,
    rooms: [room],
    sensory_havens: [],
    hallway_matches: [],
    handshakes: [],
    scored_talks: [],
    schedule: { slots: [] },
    last_elastic_query_dsl: {},
  }) as unknown as StateResponse;
const respond = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
beforeEach(() => {
  setActivePinia(createPinia());
  vi.restoreAllMocks();
  vi.stubGlobal(
    "EventSource",
    class {
      onopen = null;
      onerror = null;
      onmessage = null;
      close() {}
    },
  );
});
describe("live radar reconciliation", () => {
  it("scopes and deduplicates saved handshakes", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        respond({
          ...state(),
          handshakes: [record("1"), record("2", "bob")],
        }),
      ),
    );
    const store = useVectorStore();
    await store.hydrate();
    const event = (h: HandshakeRecord): SseEvent => ({
      event_type: "handshake_completed",
      data: {
        handshake: h,
        trace_id: "trace",
        timestamp_utc: "2026-11-12T21:00:00Z",
      },
    });
    store.handleEvent(event(record("1")));
    store.handleEvent(event(record("3", "bob")));
    expect(store.handshakes.map((h) => h.handshake_id)).toEqual(["1"]);
  });
  it("applies live capacity and clears a reset surge", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(respond(state())));
    const store = useVectorStore();
    await store.hydrate();
    store.activeSurgeAlert = {
      jammed_room_id: "ballroom-b",
    } as typeof store.activeSurgeAlert;
    store.handleEvent({
      event_type: "room_telemetry_update",
      data: {
        room: { ...room, capacity_current: 142, occupancy_ratio: 142 / 150 },
        trace_id: "live",
        timestamp_utc: "2026-11-12T21:00:00Z",
      },
    });
    expect(store.rooms[0].capacity_current).toBe(142);
    expect(store.activeSurgeAlert).toBeNull();
  });
  it("preserves newer live readings over a stale hydration response", async () => {
    let finish!: (r: Response) => void;
    vi.stubGlobal(
      "fetch",
      vi.fn().mockReturnValue(
        new Promise<Response>((resolve) => {
          finish = resolve;
        }),
      ),
    );
    const store = useVectorStore(),
      pending = store.hydrate();
    store.handleEvent({
      event_type: "room_telemetry_update",
      data: {
        room: { ...room, capacity_current: 150, occupancy_ratio: 1 },
        trace_id: "new",
        timestamp_utc: "2026-11-12T21:00:00Z",
      },
    });
    finish(
      respond({
        ...state(),
        rooms: [room, { ...room, room_id: "room-250a" }],
      }),
    );
    await pending;
    expect(store.rooms[0].capacity_current).toBe(150);
    expect(store.rooms.map((r) => r.room_id)).toEqual([
      "ballroom-b",
      "room-250a",
    ]);
  });
  it("surfaces errors and releases controls for retry", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        respond(
          {
            error: {
              message: "Elasticsearch unavailable",
              trace_id: "error-trace",
            },
          },
          502,
        ),
      ),
    );
    const store = useVectorStore();
    await store.refresh();
    expect(store.error).toBe("Elasticsearch unavailable");
    expect(store.busy).toBe(false);
    expect(store.loading).toBe(false);
  });
  it("bounds the event log", () => {
    const store = useVectorStore();
    for (let n = 0; n < 30; n++)
      store.handleEvent({
        event_type: "room_telemetry_update",
        data: {
          room,
          trace_id: String(n),
          timestamp_utc: "2026-11-12T21:00:00Z",
        },
      });
    expect(store.sseEventLog).toHaveLength(25);
    expect(store.lastTraceId).toBe("29");
  });
});

describe("session lifecycle", () => {
  it("clears private attendee state on an expired cookie", async () => {
    const fetch = vi
      .fn()
      .mockResolvedValueOnce(respond(state()))
      .mockResolvedValueOnce(
        respond({ error: { message: "Sign in required" } }, 401),
      );
    vi.stubGlobal("fetch", fetch);
    const store = useVectorStore();
    await store.hydrate();
    store.isVaultOpen = true;
    store.isQrModalOpen = true;
    expect(store.authenticated).toBe(true);
    await store.refresh();
    expect(store.authenticated).toBe(false);
    expect(store.profile).toBeNull();
    expect(store.rooms).toEqual([]);
    expect(store.handshakes).toEqual([]);
    expect(store.isVaultOpen).toBe(false);
    expect(store.isQrModalOpen).toBe(false);
    expect(store.sseConnected).toBe(false);
  });
  it("rejects a hydration response that arrives after sign-out", async () => {
    let finish!: (response: Response) => void;
    vi.stubGlobal(
      "fetch",
      vi.fn().mockReturnValue(
        new Promise<Response>((resolve) => {
          finish = resolve;
        }),
      ),
    );
    const store = useVectorStore();
    const pending = store.hydrate();
    store.clearSession();
    finish(respond(state()));
    await pending;
    expect(store.authenticated).toBe(false);
    expect(store.profile).toBeNull();
  });
});
