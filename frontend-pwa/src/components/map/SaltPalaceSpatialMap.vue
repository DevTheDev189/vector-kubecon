<script setup lang="ts">
import { computed } from "vue";
import { Maximize2, Minimize2, Navigation } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
import type { RoomState } from "../../types/vector";
const store = useVectorStore();
const destination = computed(() =>
  store.quietMode
    ? store.haven
    : store.rooms.find(
        (r) =>
          r.room_id ===
          store.activeSurgeAlert?.recommended_alternative?.room_id,
      ),
);
const origin = computed(() =>
  store.activeSurgeAlert
    ? store.rooms.find(
        (r) => r.room_id === store.activeSurgeAlert?.jammed_room_id,
      )
    : store.currentRoom,
);
const path = computed(() => {
  const a = origin.value?.map_point,
    b = destination.value?.map_point;
  return a && b && (a.x !== b.x || a.y !== b.y)
    ? `M ${a.x} ${a.y} L ${a.x} 220 L ${b.x} 220 L ${b.x} ${b.y}`
    : "";
});
const isHaven = (r: RoomState) =>
  store.sensoryHavens.some((h) => h.room_id === r.room_id) &&
  !r.active_talk_id &&
  r.occupancy_ratio < 0.2 &&
  r.decibel_level < 45;
const tone = (r: RoomState) =>
  isHaven(r)
    ? "haven"
    : r.occupancy_ratio >= 0.95
      ? "full"
      : r.occupancy_ratio >= 0.7
        ? "busy"
        : "open";
const short = (r: RoomState) =>
  r.room_name
    .replace("Grand ", "")
    .replace(" (Alpine Room)", "")
    .replace(" (Sensory Haven)", "")
    .replace("Salt Palace ", "");
</script>
<template>
  <section class="spatial-map" :class="{ expanded: store.isMapExpanded }">
    <div class="map-heading">
      <span class="eyebrow"><Navigation :size="12" /> SALT PALACE / LIVE</span
      ><button
        class="text-button"
        :aria-expanded="store.isMapExpanded"
        @click="store.isMapExpanded = !store.isMapExpanded"
      >
        <Minimize2 v-if="store.isMapExpanded" :size="12" /><Maximize2
          v-else
          :size="12"
        />
        {{ store.isMapExpanded ? "Collapse" : "Expand" }}
      </button>
    </div>
    <svg
      viewBox="0 0 800 540"
      role="group"
      aria-label="Salt Palace room occupancy map"
    >
      <defs>
        <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r=".8" fill="#263443" />
        </pattern>
        <filter id="glow"><feGaussianBlur stdDeviation="4" /></filter>
      </defs>
      <rect width="800" height="540" fill="url(#grid)" />
      <path
        d="M30 35H770V505H30Z M30 220H770 M30 380H770 M220 35V505 M405 35V505 M575 35V505"
        fill="none"
        stroke="#263342"
        stroke-width="2"
      />
      <text x="40" y="22" class="map-wing">NORTH WING</text>
      <text x="40" y="535" class="map-wing">SOUTH CONCOURSE</text>
      <text x="750" y="22" text-anchor="end" class="map-wing">N ↑</text>
      <g
        v-for="room in store.rooms"
        :key="room.room_id"
        :transform="`translate(${room.map_point.x},${room.map_point.y})`"
        class="map-room"
        :class="[
          tone(room),
          { selected: store.selectedRoom?.room_id === room.room_id },
        ]"
        role="button"
        tabindex="0"
        :aria-label="`${room.room_name}, ${Math.round(room.occupancy_ratio * 100)} percent occupied, ${room.decibel_level} decibels`"
        @click="store.selectedRoomId = room.room_id"
        @keydown.enter.prevent="store.selectedRoomId = room.room_id"
        @keydown.space.prevent="store.selectedRoomId = room.room_id"
      >
        <rect x="-72" y="-54" width="144" height="108" rx="9" />
        <text y="-15" text-anchor="middle" class="map-room-name">
          {{ short(room).slice(0, 21) }}
        </text>
        <text y="13" text-anchor="middle" class="map-room-value">
          {{
            isHaven(room)
              ? "QUIET"
              : Math.round(room.occupancy_ratio * 100) + "%"
          }}
        </text>
        <text y="35" text-anchor="middle" class="map-room-detail">
          {{ room.capacity_current }}/{{ room.capacity_max }} ·
          {{ room.decibel_level.toFixed(0) }} dB
        </text>
      </g>
      <path
        v-if="path"
        :d="path"
        fill="none"
        :stroke="store.quietMode ? '#2dd4bf' : '#b7f769'"
        stroke-width="10"
        opacity=".3"
        filter="url(#glow)"
      />
      <path
        v-if="path"
        :d="path"
        class="reroute-path"
        fill="none"
        :stroke="store.quietMode ? '#2dd4bf' : '#b7f769'"
        stroke-width="4"
        stroke-dasharray="6,6"
      />
      <g
        v-if="store.currentRoom"
        :transform="`translate(${store.currentRoom.map_point.x - 65},${store.currentRoom.map_point.y - 50})`"
      >
        <circle r="9" fill="#fff" stroke="#07090e" stroke-width="4" />
      </g>
    </svg>
    <div class="map-legend">
      <span><i class="legend-dot open" /> Open</span
      ><span><i class="legend-dot busy" /> Filling</span
      ><span><i class="legend-dot full" /> ≥95%</span
      ><span><i class="legend-dot haven" /> Quiet haven</span>
    </div>
    <div
      v-if="store.selectedRoomId && store.selectedRoom"
      class="map-selection"
    >
      <strong>{{ store.selectedRoom.room_name }}</strong
      ><span>{{
        store.selectedRoom.active_talk_title || "No active talk"
      }}</span
      ><button class="text-button" @click="store.selectedRoomId = ''">
        Follow my route ↗
      </button>
    </div>
    <div class="map-caption">
      <span class="status-dot" />
      {{
        store.quietMode
          ? "Finding a little breathing room."
          : store.activeSurgeAlert
            ? "A better route. Already found."
            : "Your next move, in real time."
      }}
    </div>
  </section>
</template>
