import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { ApiError, apiFetch, requestTelemetry, send } from "../api/client";
import type {
  BadgeResponse,
  BuilderProfile,
  CustomSchedule,
  DnaProgress,
  HandshakeRecord,
  MatchedBuilder,
  RoomState,
  ScoredTalk,
  SseEvent,
  StateResponse,
  SurgeAlert,
} from "../types/vector";

export const localTime = (value: string) =>
  new Intl.DateTimeFormat("en-US", {
    timeZone: "America/Denver",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(value));
export const useVectorStore = defineStore("vector", () => {
  const profile = ref<BuilderProfile | null>(null),
    schedule = ref<CustomSchedule | null>(null);
  const scoredTalks = ref<ScoredTalk[]>([]),
    rooms = ref<RoomState[]>([]),
    sensoryHavens = ref<RoomState[]>([]);
  const hallwayMatches = ref<MatchedBuilder[]>([]),
    handshakes = ref<HandshakeRecord[]>([]);
  const selectedSlotIndex = ref(2),
    selectedRoomId = ref(""),
    isMapExpanded = ref(false),
    isVaultOpen = ref(false),
    isQrModalOpen = ref(false);
  const activeSurgeAlert = ref<SurgeAlert | null>(null),
    lastElasticDsl = ref<unknown>(null);
  const sseConnected = ref(false),
    sseEventLog = ref<SseEvent[]>([]),
    dnaSyncSteps = ref<DnaProgress[]>([]);
  const authenticated = ref(false),
    oauthConfigured = ref<boolean | null>(null),
    canControlDemo = ref(false);
  const busy = ref(false),
    loading = ref(true),
    error = ref(""),
    syncUsername = ref(""),
    qrPeer = ref("");
  const badge = ref<BadgeResponse | null>(null),
    health = ref<"unknown" | "ok" | "unavailable">("unknown");
  let stream: EventSource | null = null,
    hydrationVersion = 0,
    roomRevision = 0,
    hasConnected = false;
  const liveRoomRevisions = new Map<string, number>();
  const lastTraceId = computed(() => requestTelemetry.lastTraceId);
  const quietMode = computed(
    () => profile.value?.sensory_prefs.sensory_mode_enabled ?? false,
  );
  const slots = computed(() => schedule.value?.slots ?? []);
  const selectedSlot = computed(() => slots.value[selectedSlotIndex.value]);
  const activeTalk = computed(() => {
    const alternative = activeSurgeAlert.value?.recommended_alternative;
    if (
      alternative &&
      alternative.start_time === selectedSlot.value?.start_time
    )
      return alternative;
    return selectedSlot.value?.primary_talk ?? null;
  });
  const selectedRoom = computed(() =>
    rooms.value.find(
      (r) => r.room_id === (selectedRoomId.value || activeTalk.value?.room_id),
    ),
  );
  const currentRoom = computed(() =>
    rooms.value.find((r) => r.room_id === profile.value?.current_room_id),
  );
  const haven = computed(() => sensoryHavens.value[0] ?? null);
  const visiblePeers = computed(() =>
    (
      activeSurgeAlert.value?.bumped_hallway_peers ?? hallwayMatches.value
    ).slice(0, 3),
  );
  function mergeRoom(room: RoomState) {
    roomRevision++;
    liveRoomRevisions.set(room.room_id, roomRevision);
    const index = rooms.value.findIndex((r) => r.room_id === room.room_id);
    if (index < 0) rooms.value.push(room);
    else rooms.value[index] = room;
    // Reconcile room telemetry with every visible copy of its active talk.
    const update = (talk: ScoredTalk | null) => {
      if (talk?.talk_id === room.active_talk_id)
        Object.assign(talk, {
          capacity_current: room.capacity_current,
          occupancy_ratio: room.occupancy_ratio,
          decibel_level: room.decibel_level,
        });
    };
    scoredTalks.value.forEach(update);
    slots.value.forEach((slot) => {
      update(slot.primary_talk);
      update(slot.backup_talk);
    });
    update(activeSurgeAlert.value?.recommended_alternative ?? null);
    sensoryHavens.value = sensoryHavens.value
      .filter(
        (r) =>
          r.room_id !== room.room_id ||
          (room.occupancy_ratio < 0.2 &&
            room.decibel_level < 45 &&
            !room.active_talk_id),
      )
      .map((r) => (r.room_id === room.room_id ? room : r));
    if (
      !room.active_talk_id &&
      room.decibel_level < 45 &&
      room.occupancy_ratio < 0.2 &&
      !sensoryHavens.value.some((haven) => haven.room_id === room.room_id)
    ) {
      sensoryHavens.value.push(room);
    }
    if (
      activeSurgeAlert.value?.jammed_room_id === room.room_id &&
      room.occupancy_ratio < 0.98
    )
      activeSurgeAlert.value = null;
  }
  function saveHandshake(record: HandshakeRecord) {
    if (record.initiator_username !== profile.value?.username) return;
    if (!handshakes.value.some((h) => h.handshake_id === record.handshake_id))
      handshakes.value.unshift(record);
  }
  function handleEvent(event: SseEvent) {
    if (!event || typeof event !== "object" || !event.data)
      throw new Error("Invalid live event");
    sseEventLog.value = [event, ...sseEventLog.value].slice(0, 25);
    if ("trace_id" in event.data)
      requestTelemetry.lastTraceId = event.data.trace_id;
    switch (event.event_type) {
      case "room_telemetry_update":
        mergeRoom(event.data.room);
        break;
      case "crowd_surge_alert":
        activeSurgeAlert.value = event.data;
        {
          const index = slots.value.findIndex(
            (s) =>
              s.start_time === event.data.recommended_alternative?.start_time,
          );
          if (index >= 0) selectedSlotIndex.value = index;
        }
        selectedRoomId.value = "";
        break;
      case "dna_sync_progress":
        if (event.data.username === syncUsername.value)
          dnaSyncSteps.value = [
            ...dnaSyncSteps.value.filter((s) => s.step !== event.data.step),
            event.data,
          ];
        break;
      case "handshake_completed":
        saveHandshake(event.data.handshake);
        break;
      case "sensory_haven_update":
        sensoryHavens.value = [
          event.data.recommended_haven,
          ...sensoryHavens.value.filter(
            (r) => r.room_id !== event.data.recommended_haven.room_id,
          ),
        ];
        break;
    }
  }
  async function hydrate() {
    const version = ++hydrationVersion,
      revision = roomRevision;
    const data = await apiFetch<StateResponse>("/state");
    if (version !== hydrationVersion) return;
    authenticated.value = true;
    canControlDemo.value = data.can_control_demo ?? false;
    profile.value = data.profile;
    schedule.value = data.schedule;
    scoredTalks.value = data.scored_talks;
    const newerRooms = rooms.value.filter(
      (room) => (liveRoomRevisions.get(room.room_id) ?? 0) > revision,
    );
    rooms.value = data.rooms;
    sensoryHavens.value = data.sensory_havens;
    hallwayMatches.value = data.hallway_matches;
    handshakes.value = data.handshakes.filter(
      (h) => h.initiator_username === data.profile.username,
    );
    newerRooms.forEach(mergeRoom);
    lastElasticDsl.value = data.last_elastic_query_dsl;
    connectSseStream();
    selectedSlotIndex.value = Math.min(
      selectedSlotIndex.value,
      Math.max(0, slots.value.length - 1),
    );
  }
  async function run(action: () => Promise<void>) {
    if (busy.value) return;
    busy.value = true;
    error.value = "";
    try {
      await action();
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 401) {
        clearSession();
        return;
      }
      error.value =
        cause instanceof Error
          ? cause.message
          : "Something went wrong. Please retry.";
    } finally {
      busy.value = false;
    }
  }
  async function refresh() {
    loading.value = true;
    await run(hydrate);
    loading.value = false;
  }
  function connectSseStream() {
    if (stream || !authenticated.value) return;
    stream = new EventSource("/api/events/stream");
    stream.onopen = () => {
      sseConnected.value = true;
      if (hasConnected && !busy.value) void run(hydrate);
      hasConnected = true;
    };
    stream.onerror = () => {
      sseConnected.value = false;
      // Re-check the cookie when a stream expires or the server rejects it.
      void apiFetch("/me").catch((cause) => {
        if (cause instanceof ApiError && cause.status === 401) clearSession();
      });
    };
    stream.onmessage = (message) => {
      try {
        handleEvent(JSON.parse(message.data) as SseEvent);
      } catch {
        error.value = "A live update could not be read. Refresh to resync.";
      }
    };
  }
  function disconnect() {
    stream?.close();
    stream = null;
    sseConnected.value = false;
  }
  function clearSession() {
    hydrationVersion++;
    disconnect();
    authenticated.value = false;
    canControlDemo.value = false;
    profile.value = null;
    schedule.value = null;
    rooms.value = [];
    scoredTalks.value = [];
    sensoryHavens.value = [];
    hallwayMatches.value = [];
    handshakes.value = [];
    activeSurgeAlert.value = null;
    lastElasticDsl.value = null;
    badge.value = null;
    qrPeer.value = "";
    selectedRoomId.value = "";
    isQrModalOpen.value = false;
    isVaultOpen.value = false;
    dnaSyncSteps.value = [];
    syncUsername.value = "";
    sseEventLog.value = [];
    liveRoomRevisions.clear();
    requestTelemetry.lastTraceId = "";
    hasConnected = false;
  }
  async function initialize() {
    try {
      const status = await apiFetch<{ oauth_configured: boolean }>(
        "/auth/status",
      );
      oauthConfigured.value = status.oauth_configured;
    } catch {
      oauthConfigured.value = null;
    }
    await refresh();
  }
  async function logout() {
    await run(async () => {
      await apiFetch("/auth/logout", send("POST"));
      clearSession();
    });
  }
  async function sync() {
    if (!profile.value) return;
    await run(async () => {
      await apiFetch(
        "/sync-github",
        send("POST", { username: profile.value!.username }),
      );
      await hydrate();
    });
  }
  async function toggleQuiet() {
    await run(async () => {
      await apiFetch(
        "/sensory",
        send("PATCH", { sensory_mode_enabled: !quietMode.value }),
      );
      await hydrate();
    });
  }
  async function triggerCrowdSurge() {
    await run(async () => {
      const data = await apiFetch<{
        room: RoomState;
        surge: Extract<SseEvent, { event_type: "crowd_surge_alert" }>;
        elastic_query_dsl: unknown;
      }>("/surge", send("POST", { room_id: "ballroom-b" }));
      mergeRoom(data.room);
      handleEvent(data.surge);
      lastElasticDsl.value = data.elastic_query_dsl;
    });
  }
  async function resetSurge() {
    await run(async () => {
      const data = await apiFetch<{ room: RoomState }>(
        "/surge/reset",
        send("POST"),
      );
      mergeRoom(data.room);
      activeSurgeAlert.value = null;
      await hydrate();
    });
  }
  async function sampleNoise() {
    await run(async () => {
      const room = currentRoom.value;
      if (!room) throw new Error("Select a profile first.");
      const data = await apiFetch<{ room: RoomState }>(
        "/rooms/checkin",
        send("POST", {
          room_id: room.room_id,
          delta: 0,
          decibel_reading: room.decibel_level,
        }),
      );
      mergeRoom(data.room);
    });
  }
  async function openBadge(peer = "") {
    qrPeer.value = peer;
    badge.value = null;
    isQrModalOpen.value = true;
    await run(async () => {
      badge.value = await apiFetch<BadgeResponse>("/handshake/badge");
    });
  }
  async function connectPeer(username: string, pin: string) {
    await run(async () => {
      if (!username.trim() || username === profile.value?.username)
        throw new Error("Enter another attendee’s GitHub handle.");
      const data = await apiFetch<{ handshake: HandshakeRecord }>(
        "/handshake/connect",
        send("POST", {
          peer_username: username.trim(),
          pin_code: pin || null,
          room_id: currentRoom.value?.room_id,
          talk_title: activeTalk.value?.title,
        }),
      );
      saveHandshake(data.handshake);
      isQrModalOpen.value = false;
      isVaultOpen.value = true;
    });
  }
  async function bookmark(talk: ScoredTalk) {
    await run(async () => {
      await apiFetch(
        "/schedule/bookmark",
        send("POST", { talk_id: talk.talk_id }),
      );
      await hydrate();
    });
  }
  async function checkHealth() {
    try {
      const data = await apiFetch<{ status: string }>("/health");
      health.value = data.status === "ok" ? "ok" : "unavailable";
    } catch {
      health.value = "unavailable";
    }
  }
  return {
    authenticated,
    oauthConfigured,
    canControlDemo,
    initialize,
    logout,
    clearSession,
    profile,
    schedule,
    scoredTalks,
    rooms,
    sensoryHavens,
    hallwayMatches,
    handshakes,
    selectedSlotIndex,
    selectedRoomId,
    isMapExpanded,
    isVaultOpen,
    isQrModalOpen,
    activeSurgeAlert,
    lastElasticDsl,
    lastTraceId,
    sseConnected,
    sseEventLog,
    dnaSyncSteps,
    syncUsername,
    busy,
    loading,
    error,
    quietMode,
    slots,
    selectedSlot,
    activeTalk,
    selectedRoom,
    currentRoom,
    haven,
    visiblePeers,
    badge,
    qrPeer,
    health,
    handleEvent,
    hydrate,
    refresh,
    connectSseStream,
    disconnect,
    sync,
    toggleQuiet,
    triggerCrowdSurge,
    resetSurge,
    sampleNoise,
    openBadge,
    connectPeer,
    bookmark,
    checkHealth,
  };
});
