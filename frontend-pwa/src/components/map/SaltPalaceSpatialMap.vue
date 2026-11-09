<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import {
  Maximize2,
  Minimize2,
  Plus,
  Minus,
  RotateCcw,
  MoveUpRight,
  Layers,
  Search,
  Leaf,
  X,
} from "lucide-vue-next";
import type { RoomState } from "../../types/vector";
import { useVectorStore } from "../../stores/vectorStore";
import {
  floorNames,
  roomSpace,
  spaces,
  type Floor,
  type MapSpace,
} from "../../map/saltPalace";
const store = useVectorStore();
const floor = ref<Floor>(1),
  zoom = ref(1),
  offset = ref({ x: 0, y: 0 });
const searchQuery = ref(""),
  quietOnly = ref(false),
  finderOpen = ref(false);
const searchInput = ref<HTMLInputElement>(),
  finderButton = ref<HTMLButtonElement>(),
  searchPanel = ref<HTMLElement>(),
  mapSection = ref<HTMLElement>();
function openFinder(quiet = false) {
  finderOpen.value = true;
  quietOnly.value = quiet;
  searchQuery.value = "";
  void nextTick(() =>
    quiet
      ? searchPanel.value
          ?.querySelector<HTMLButtonElement>(".room-search-results button")
          ?.focus()
      : searchInput.value?.focus(),
  );
}
function closeFinder(restoreFocus = true) {
  finderOpen.value = false;
  searchQuery.value = "";
  quietOnly.value = false;
  if (restoreFocus) void nextTick(() => finderButton.value?.focus());
}
const isQuiet = (room: RoomState) =>
  !room.active_talk_id && room.decibel_level < 45 && room.occupancy_ratio < 0.2;
const quietRooms = computed(() =>
  store.rooms.filter(isQuiet).sort((a, b) => a.decibel_level - b.decibel_level),
);
const quietOnFloor = (level: Floor) =>
  quietRooms.value.filter((room) => roomSpace(room.room_id)?.floor === level);
const searchResults = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  const pool = quietOnly.value ? quietRooms.value : store.rooms;
  return pool
    .filter((room) => {
      const space = roomSpace(room.room_id);
      return (
        space &&
        `${room.room_name} ${room.room_id} ${room.active_talk_title || ""} ${floorNames[space.floor]}`
          .toLowerCase()
          .includes(query)
      );
    })
    .sort((a, b) => a.room_name.localeCompare(b.room_name))
    .slice(0, 8);
});
function focusRoom(roomId: string) {
  store.inspectRoomOnMap(roomId);
}
const svg = ref<SVGSVGElement>();
let drag: { x: number; y: number; ox: number; oy: number } | null = null;
const destination = computed(() =>
  roomSpace(
    store.quietMode
      ? store.haven?.room_id
      : store.activeSurgeAlert?.recommended_alternative?.room_id,
  ),
);
const origin = computed(() =>
  roomSpace(
    store.activeSurgeAlert?.jammed_room_id || store.currentRoom?.room_id,
  ),
);
const focused = computed(() =>
  roomSpace(
    store.selectedRoomId ||
      store.activeTalk?.room_id ||
      (store.selectedSlot?.is_sensory_break
        ? store.haven?.room_id
        : destination.value?.roomId),
  ),
);
watch(
  () => focused.value?.floor,
  (value) => {
    if (value) {
      floor.value = value;
      resetView();
    }
  },
);
watch(
  () => store.mapFocusVersion,
  () => {
    closeFinder(false);
    const target = focused.value;
    if (target) floor.value = target.floor;
    resetView();
    void nextTick(() => {
      mapSection.value?.scrollIntoView({
        behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches
          ? "auto"
          : "smooth",
        block: "start",
      });
      svg.value?.focus({ preventScroll: true });
    });
  },
);
const inspectingRoom = computed(
  () =>
    !!store.selectedRoomId &&
    store.selectedRoomId !==
      (store.activeTalk?.room_id ||
        (store.selectedSlot?.is_sensory_break ? store.haven?.room_id : "")),
);
const displayed = computed(() =>
  spaces.filter((space) => space.floor === floor.value),
);
const room = (space: MapSpace) =>
  store.rooms.find((r) => r.room_id === space.roomId);
const status = (space: MapSpace) => {
  const live = room(space);
  if (!live) return "untracked";
  if (
    !live.active_talk_id &&
    live.decibel_level < 45 &&
    live.occupancy_ratio < 0.2
  )
    return "quiet";
  return live.occupancy_ratio >= 0.95
    ? "full"
    : live.occupancy_ratio >= 0.7
      ? "busy"
      : "open";
};
const label = (space: MapSpace) => {
  const live = room(space);
  return `${live?.room_name || space.label}${live ? `, ${Math.round(live.occupancy_ratio * 100)} percent occupied, ${live.decibel_level} decibels` : ", no live telemetry"}`;
};
function select(space: MapSpace) {
  if (space.roomId) store.selectedRoomId = space.roomId;
}
function resetView() {
  zoom.value = 1;
  offset.value = { x: 0, y: 0 };
}
function changeZoom(delta: number) {
  zoom.value = Math.max(1, Math.min(3, zoom.value + delta));
  if (zoom.value === 1) offset.value = { x: 0, y: 0 };
}
const viewBox = computed(
  () =>
    `${500 - 500 / zoom.value + offset.value.x} ${440 - 440 / zoom.value + offset.value.y} ${1000 / zoom.value} ${880 / zoom.value}`,
);
function pointerDown(event: PointerEvent) {
  if (zoom.value <= 1 || (event.target as Element).closest('[role="button"]'))
    return;
  drag = {
    x: event.clientX,
    y: event.clientY,
    ox: offset.value.x,
    oy: offset.value.y,
  };
  svg.value?.setPointerCapture(event.pointerId);
}
function pointerMove(event: PointerEvent) {
  if (!drag || !svg.value) return;
  const bounds = svg.value.getBoundingClientRect(),
    scale = Math.min(
      bounds.width / (1000 / zoom.value),
      bounds.height / (880 / zoom.value),
    );
  offset.value = {
    x: Math.max(
      -450,
      Math.min(450, drag.ox - (event.clientX - drag.x) / scale),
    ),
    y: Math.max(
      -390,
      Math.min(390, drag.oy - (event.clientY - drag.y) / scale),
    ),
  };
}
const visibleOrigin = computed(() =>
  origin.value?.floor === floor.value ? origin.value : null,
);
</script>
<template>
  <section
    ref="mapSection"
    class="spatial-map architectural-map"
    :class="{ expanded: store.isMapExpanded }"
  >
    <div class="map-heading">
      <div>
        <span class="eyebrow"><Layers :size="13" /> SALT PALACE</span>
        <p class="map-subtitle">
          {{
            store.activeTalk
              ? "Selected session · " +
                store.activeTalk.room_name.replace(/\s*\(.*/, "")
              : "Explore the conference spaces."
          }}
        </p>
      </div>
      <button
        class="icon-button"
        :aria-expanded="store.isMapExpanded"
        :aria-label="store.isMapExpanded ? 'Collapse map' : 'Expand map'"
        @click="store.isMapExpanded = !store.isMapExpanded"
      >
        <Minimize2 v-if="store.isMapExpanded" :size="16" /><Maximize2
          v-else
          :size="16"
        />
      </button>
    </div>
    <div v-if="store.authenticated" class="map-discovery-actions">
      <button
        ref="finderButton"
        :aria-expanded="finderOpen"
        aria-controls="room-finder-panel"
        @click="finderOpen ? closeFinder() : openFinder()"
      >
        <Search :size="15" /> Find your room
      </button>
      <button
        class="quiet-spaces-shortcut"
        :aria-expanded="finderOpen && quietOnly"
        aria-controls="room-finder-panel"
        @click="openFinder(true)"
      >
        <Leaf :size="15" /> Quiet spaces <span>{{ quietRooms.length }}</span>
      </button>
    </div>
    <div
      v-if="finderOpen"
      id="room-finder-panel"
      ref="searchPanel"
      class="room-discovery"
      @keydown.esc.stop.prevent="closeFinder()"
    >
      <div class="finder-panel-heading">
        <strong>Find a room</strong
        ><button
          class="icon-button"
          aria-label="Close room search"
          @click="closeFinder()"
        >
          <X :size="16" />
        </button>
      </div>
      <label class="room-search-label" for="room-search"
        ><Search :size="17" /> Find your room</label
      >
      <div class="room-search-field">
        <Search :size="18" /><input
          id="room-search"
          ref="searchInput"
          v-model="searchQuery"
          type="search"
          placeholder="Room number, hall, or session…"
          autocomplete="off"
          :aria-expanded="!!searchQuery.trim() || quietOnly"
          aria-controls="room-search-results"
        /><button
          v-if="searchQuery"
          aria-label="Clear room search"
          @click="searchQuery = ''"
        >
          <X :size="16" />
        </button>
      </div>
      <button
        class="quiet-discovery-button"
        :aria-pressed="quietOnly"
        @click="quietOnly = !quietOnly"
      >
        <Leaf :size="18" /><span
          ><strong>Find a quiet space</strong
          ><small>{{
            store.authenticated
              ? `${quietRooms.length} available across all floors`
              : "Sign in for live availability"
          }}</small></span
        ><span class="quiet-count">{{
          store.authenticated ? quietRooms.length : "—"
        }}</span>
      </button>
      <div
        v-if="searchQuery.trim() || quietOnly"
        id="room-search-results"
        class="room-search-results"
        aria-label="Room search results"
      >
        <button
          v-for="result in searchResults"
          :key="result.room_id"
          @click="focusRoom(result.room_id)"
        >
          <Leaf v-if="isQuiet(result)" :size="16" /><Search
            v-else
            :size="16"
          /><span
            ><strong>{{ result.room_name }}</strong
            ><small
              >{{ floorNames[roomSpace(result.room_id)!.floor] }} ·
              {{ result.decibel_level.toFixed(1) }} dB{{
                isQuiet(result) ? " · Quiet space" : ""
              }}</small
            ></span
          ><MoveUpRight :size="16" />
        </button>
        <p v-if="!searchResults.length" role="status">
          No rooms match. Try a room number or another keyword.
        </p>
      </div>
    </div>
    <div class="floor-switcher" aria-label="Map floor">
      <button
        v-for="level in [1, 2, 3] as Floor[]"
        :key="level"
        :aria-pressed="floor === level"
        @click="
          floor = level;
          resetView();
        "
      >
        <span class="mono">0{{ level }}</span> {{ floorNames[level]
        }}<i
          v-if="quietOnFloor(level).length"
          class="floor-quiet-dot"
          aria-hidden="true"
          :title="`${quietOnFloor(level).length} quiet spaces`"
        />
      </button>
    </div>
    <div class="map-viewport">
      <svg
        ref="svg"
        tabindex="-1"
        :viewBox="viewBox"
        role="group"
        :aria-label="`Salt Palace architectural map, ${floorNames[floor]}`"
        :class="{ zoomed: zoom > 1 }"
        @pointerdown="pointerDown"
        @pointermove="pointerMove"
        @pointerup="drag = null"
        @pointercancel="drag = null"
        @lostpointercapture="drag = null"
      >
        <defs>
          <pattern
            id="map-dots"
            width="18"
            height="18"
            patternUnits="userSpaceOnUse"
          >
            <circle cx="1" cy="1" r=".7" fill="#343434" />
          </pattern>
          <pattern
            id="capacity-hatch"
            width="8"
            height="8"
            patternUnits="userSpaceOnUse"
          >
            <path
              d="M-2 2L2-2M0 8L8 0M6 10L10 6"
              stroke="#888"
              stroke-width="1"
            />
          </pattern>
        </defs>
        <rect width="1000" height="880" fill="url(#map-dots)" />
        <g class="building-context" aria-hidden="true">
          <path
            class="building-outline"
            d="M44 467H465V133H560V50H702V610H575L545 653H402V817H192V717H44Z"
          />
          <path
            class="concourse"
            d="M52 650H527L560 604H700V470H706M190 701H580Q685 701 675 798M707 650H858V162H892V62H858"
          />
          <path class="loading-zone" d="M60 428H468V184H447V428H60Z" />
          <path
            class="building-outline"
            d="M800 150H893V477H998V600H893V833H680V688H800Z"
          />
          <path
            class="stairs"
            d="M650 621h22m-22 7h22m-22 7h22m-22 7h22m-22 7h22M864 480h22m-22 7h22m-22 7h22m-22 7h22"
          />
          <text x="55" y="410" class="architecture-note">
            LOADING / SERVICE
          </text>
          <text x="300" y="675" class="architecture-note" text-anchor="middle">
            LOWER CONCOURSE
          </text>
          <text
            x="645"
            y="732"
            class="architecture-note"
            transform="rotate(-90 645 732)"
          >
            UPPER CONCOURSE
          </text>
          <text x="288" y="842" class="architecture-note" text-anchor="middle">
            BALLROOM
          </text>
          <text x="95" y="733" class="architecture-note">SOUTH</text>
          <text x="95" y="754" class="architecture-note">ENTRANCE ↗</text>
          <text x="578" y="38" class="architecture-note">WEST LOBBY</text>
          <text x="818" y="125" class="architecture-note">MEETING ROOMS</text>
        </g>
        <g
          v-for="space in displayed"
          :key="space.id"
          class="map-room"
          :class="[
            space.kind,
            status(space),
            { selected: focused?.id === space.id, tracked: !!room(space) },
          ]"
          :role="space.roomId ? 'button' : undefined"
          :tabindex="space.roomId ? 0 : undefined"
          :aria-label="label(space)"
          :aria-pressed="space.roomId ? focused?.id === space.id : undefined"
          :data-room-id="space.roomId"
          @click="select(space)"
          @keydown.enter.prevent="select(space)"
          @keydown.space.prevent="select(space)"
        >
          <title>{{ label(space) }}</title>
          <path class="room-shape" :d="space.d" />
          <path
            v-if="status(space) === 'full'"
            class="room-hatch"
            :d="space.d"
            fill="url(#capacity-hatch)"
          />
          <text
            :x="space.x"
            :y="space.y - (room(space) ? 9 : 0)"
            class="space-label"
            text-anchor="middle"
          >
            <template v-if="/^HALL [A-E]$/.test(space.label)">
              <tspan :x="space.x" dy="-12" class="hall-prefix">HALL</tspan>
              <tspan :x="space.x" dy="28">{{ space.label.at(-1) }}</tspan>
            </template>
            <template v-else>{{ space.label }}</template>
          </text>
          <text
            v-if="room(space)"
            :x="space.x"
            :y="space.y + 15"
            class="space-reading"
            text-anchor="middle"
          >
            {{
              status(space) === "quiet"
                ? "QUIET"
                : Math.round(room(space)!.occupancy_ratio * 100) + "%"
            }}
          </text>
        </g>
        <g
          v-for="space in displayed.filter(
            (space) => room(space) && isQuiet(room(space)!),
          )"
          :key="'quiet-' + space.id"
          class="quiet-map-pin"
          :transform="`translate(${space.x},${space.y - 35})`"
          aria-hidden="true"
          pointer-events="none"
        >
          <rect x="-63" y="-22" width="126" height="36" rx="9" />
          <text text-anchor="middle" y="1">
            ♧ QUIET · {{ room(space)!.decibel_level.toFixed(0) }} dB
          </text>
          <path d="M-5 14L0 21L5 14" />
        </g>
        <g
          v-if="visibleOrigin"
          :transform="`translate(${visibleOrigin.x},${visibleOrigin.y - 34})`"
          class="you-marker"
          pointer-events="none"
        >
          <circle r="7" fill="#fff" stroke="#111" stroke-width="3" />
          <text y="-15" text-anchor="middle">YOU</text>
        </g>
        <g transform="translate(80 85)" class="north-arrow" aria-label="North">
          <path d="M0 36V0m-7 10L0 0l7 10" />
          <text x="0" y="-13" text-anchor="middle">N</text>
        </g>
      </svg>
      <div class="map-tools">
        <button
          class="icon-button"
          aria-label="Zoom out"
          :disabled="zoom <= 1"
          @click="changeZoom(-0.5)"
        >
          <Minus :size="15" /></button
        ><span class="mono">{{ Math.round(zoom * 100) }}%</span
        ><button
          class="icon-button"
          aria-label="Zoom in"
          :disabled="zoom >= 3"
          @click="changeZoom(0.5)"
        >
          <Plus :size="15" /></button
        ><button
          class="icon-button"
          aria-label="Reset map view"
          @click="resetView"
        >
          <RotateCcw :size="14" />
        </button>
      </div>
    </div>
    <div class="map-legend">
      <span><i class="legend-dot open" /> Available</span
      ><span><i class="legend-dot full" /> Full</span
      ><span><i class="legend-dot haven" /> Quiet</span
      ><span class="muted">{{ floorNames[floor] }}</span>
    </div>
    <div v-if="store.selectedRoom" class="map-selection" aria-live="polite">
      <span class="map-context-label">{{
        inspectingRoom ? "ROOM LOOKUP" : "YOUR SELECTED SESSION"
      }}</span>
      <div>
        <strong>{{ store.selectedRoom.room_name }}</strong
        ><span>{{
          store.selectedRoom.active_talk_title || "No active session"
        }}</span>
      </div>
      <div class="selection-metrics">
        <span
          >{{ Math.round(store.selectedRoom.occupancy_ratio * 100) }}%
          occupied</span
        ><span>{{ store.selectedRoom.decibel_level.toFixed(1) }} dB</span>
      </div>
    </div>
    <button
      v-if="inspectingRoom && (store.activeTalk || store.haven)"
      class="return-to-session"
      @click="store.showSessionOnMap()"
    >
      <MoveUpRight :size="15" /> Back to my session
    </button>
    <div v-if="destination" class="route-summary">
      <MoveUpRight :size="17" />
      <div>
        <strong
          >{{
            store.quietMode ? "Your quiet space" : "Your open alternative"
          }}
          · {{ destination.label }}</strong
        ><span
          >{{ floorNames[destination.floor]
          }}{{
            origin?.floor !== destination.floor
              ? " · change floors at the concourse"
              : ""
          }}</span
        >
      </div>
      <button
        @click="
          floor = destination.floor;
          resetView();
        "
      >
        Show
      </button>
    </div>
    <p class="map-caption">
      Architectural schematic · demo room placements are approximate.
    </p>
  </section>
</template>
