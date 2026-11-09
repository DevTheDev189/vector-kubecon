<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  Maximize2,
  Minimize2,
  Plus,
  Minus,
  RotateCcw,
  MoveUpRight,
  Layers,
} from "lucide-vue-next";
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
      destination.value?.roomId ||
      store.activeTalk?.room_id,
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
const visibleDestination = computed(() =>
  destination.value?.floor === floor.value ? destination.value : null,
);
// Intent line only: no assertion of a verified pedestrian or accessible route.
const routeLine = computed(() => {
  const a = visibleOrigin.value,
    b = visibleDestination.value;
  if (!a && !b) return "";
  const start = a || { x: 675, y: 670 },
    end = b || { x: 675, y: 670 };
  if (start.x === end.x && start.y === end.y) return "";
  return `M${start.x} ${start.y}Q675 670 ${end.x} ${end.y}`;
});
</script>
<template>
  <section
    class="spatial-map architectural-map"
    :class="{ expanded: store.isMapExpanded }"
  >
    <div class="map-heading">
      <div>
        <span class="eyebrow"><Layers :size="13" /> SALT PALACE</span>
        <p class="map-subtitle">A little perspective. Your next move.</p>
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
        <span class="mono">0{{ level }}</span> {{ floorNames[level] }}
      </button>
    </div>
    <div class="map-viewport">
      <svg
        ref="svg"
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
          <linearGradient
            id="map-route-gradient"
            x1="0%"
            y1="0%"
            x2="100%"
            y2="100%"
          >
            <stop offset="0%" stop-color="#22d3ee" />
            <stop offset="55%" stop-color="#f7df69" />
            <stop offset="100%" stop-color="#ef5da8" />
          </linearGradient>
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
        <path
          v-if="routeLine"
          :d="routeLine"
          class="reroute-path schematic-route"
          fill="none"
          stroke="url(#map-route-gradient)"
          stroke-width="3"
          stroke-dasharray="7 7"
          pointer-events="none"
        />
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
    <label class="map-room-picker"
      >Find a room
      <select
        aria-label="Find a room on this floor"
        :value="store.selectedRoomId"
        @change="
          store.selectedRoomId = ($event.target as HTMLSelectElement).value
        "
      >
        <option value="">Follow my session</option>
        <option
          v-for="space in displayed.filter((space) => room(space))"
          :key="space.id"
          :value="space.roomId"
        >
          {{ room(space)?.room_name }}
        </option>
      </select>
    </label>
    <div class="map-legend">
      <span><i class="legend-dot open" /> Available</span
      ><span><i class="legend-dot full" /> Full</span
      ><span><i class="legend-dot haven" /> Quiet</span
      ><span class="muted">{{ floorNames[floor] }}</span>
    </div>
    <div v-if="store.selectedRoom" class="map-selection" aria-live="polite">
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
      Architectural schematic · demo room placements are approximate.<br />Dashed
      line shows destination intent, not verified walking directions.
    </p>
  </section>
</template>
