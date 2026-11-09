<script setup lang="ts">
import { computed } from "vue";
import {
  Bookmark,
  MapPin,
  AudioLines,
  ChevronDown,
  ArrowUpRight,
  Leaf,
} from "lucide-vue-next";
import { localTime, useVectorStore } from "../../stores/vectorStore";
import { roomSpace } from "../../map/saltPalace";
import SensoryHavenBanner from "./SensoryHavenBanner.vue";
import SurgeRerouteAlert from "./SurgeRerouteAlert.vue";
const store = useVectorStore();
const dateLabel = computed(() =>
  store.slots[0]
    ? new Intl.DateTimeFormat("en-US", {
        timeZone: "America/Denver",
        weekday: "short",
        month: "short",
        day: "numeric",
      }).format(new Date(store.slots[0].start_time))
    : "Your conference day",
);
const shortRoom = (name: string) =>
  name.replace(/\s*\(.*/, "").replace(/^Grand /, "");
</script>
<template>
  <section class="day-calendar" aria-labelledby="day-heading">
    <header class="day-heading">
      <div>
        <span class="eyebrow">MADE FOR YOUR INTERESTS</span>
        <h1 id="day-heading">Your recommended day</h1>
      </div>
      <div class="day-date">
        <strong>{{ dateLabel }}</strong
        ><span>Mountain Time</span>
      </div>
    </header>
    <SurgeRerouteAlert />
    <ol
      v-if="store.slots.length"
      class="agenda-list"
      aria-label="Your recommended sessions"
    >
      <li
        v-for="(slot, index) in store.slots"
        :key="slot.start_time"
        class="agenda-row"
        :class="{
          selected: store.selectedSlotIndex === index,
          'quiet-break': slot.is_sensory_break && !store.talkForSlot(slot),
        }"
        :data-slot-index="index"
      >
        <button
          class="agenda-select"
          :aria-pressed="store.selectedSlotIndex === index"
          :aria-label="`Select ${localTime(slot.start_time)}, ${store.talkForSlot(slot)?.title || (slot.is_sensory_break ? 'Quiet break' : 'Open time')}`"
          @click="store.selectSlot(index)"
        >
          <span class="agenda-time"
            ><time :datetime="slot.start_time">{{
              localTime(slot.start_time)
            }}</time
            ><small>{{ localTime(slot.end_time) }}</small></span
          >
          <span class="agenda-session"
            ><strong>{{
              store.talkForSlot(slot)?.title ||
              (slot.is_sensory_break
                ? "A little time to recharge"
                : "Keep this time open")
            }}</strong
            ><span
              ><Leaf
                v-if="slot.is_sensory_break && !store.talkForSlot(slot)"
                :size="12"
              /><MapPin v-else :size="12" />{{
                store.talkForSlot(slot)
                  ? shortRoom(store.talkForSlot(slot)!.room_name)
                  : slot.is_sensory_break
                    ? "Quiet space"
                    : "Explore the hallway"
              }}<em v-if="store.talkForSlot(slot)?.is_rerouted_alternative"
                >New room</em
              ></span
            ></span
          >
          <Bookmark
            v-if="
              store.talkForSlot(slot) &&
              store.profile?.bookmarked_talk_ids.includes(
                store.talkForSlot(slot)!.talk_id,
              )
            "
            class="agenda-bookmark"
            :size="13"
          />
          <span
            v-if="store.selectedSlotIndex === index"
            class="agenda-selected-dot"
            aria-hidden="true"
          />
        </button>
      </li>
    </ol>
    <div v-else class="empty-state">
      <h3>Your day is taking shape.</h3>
      <p>Recommendations will appear once your profile is ready.</p>
      <button :disabled="store.busy" @click="store.refresh()">
        Refresh recommendations
      </button>
    </div>
    <div v-if="store.selectedSlot" class="session-actions">
      <details :key="store.selectedSlot.start_time" class="session-details">
        <summary>Session details <ChevronDown :size="14" /></summary>
        <article v-if="store.activeTalk" class="talk-card">
          <div class="section-heading">
            <span class="eyebrow accent">{{
              store.activeTalk.is_rerouted_alternative
                ? "YOUR OPEN ALTERNATIVE"
                : "RECOMMENDED FOR YOU"
            }}</span
            ><button
              class="icon-button"
              :aria-pressed="
                store.profile?.bookmarked_talk_ids.includes(
                  store.activeTalk.talk_id,
                )
              "
              :disabled="store.busy"
              aria-label="Bookmark session"
              @click="store.bookmark(store.activeTalk)"
            >
              <Bookmark
                :size="16"
                :fill="
                  store.profile?.bookmarked_talk_ids.includes(
                    store.activeTalk.talk_id,
                  )
                    ? 'currentColor'
                    : 'none'
                "
              />
            </button>
          </div>
          <h2>{{ store.activeTalk.title }}</h2>
          <p class="speaker">
            {{ store.activeTalk.speaker }}
            <span class="muted">/ {{ store.activeTalk.company }}</span>
          </p>
          <div class="talk-room">
            <span
              >{{ store.activeTalk.room_name }} · Floor
              {{
                roomSpace(store.activeTalk.room_id)?.floor ||
                store.activeTalk.floor
              }}</span
            ><span class="noise-badge"
              ><AudioLines :size="12" />
              {{ store.activeTalk.decibel_level.toFixed(0) }} dB</span
            >
          </div>
          <div
            class="progress-track occupancy"
            :class="{ danger: store.activeTalk.occupancy_ratio >= 0.95 }"
          >
            <i
              :style="{
                width:
                  Math.min(100, store.activeTalk.occupancy_ratio * 100) + '%',
              }"
            />
          </div>
          <div class="section-heading">
            <span class="micro muted"
              >{{ store.activeTalk.capacity_current }} /
              {{ store.activeTalk.capacity_max }} ATTENDEES</span
            ><span class="micro accent"
              >{{
                Math.max(
                  0,
                  store.activeTalk.capacity_max -
                    store.activeTalk.capacity_current,
                )
              }}
              SEATS OPEN</span
            >
          </div>
          <details class="match-details">
            <summary>Why this session?</summary>
            <p v-for="reason in store.activeTalk.match_reasons" :key="reason">
              {{ reason }}
            </p>
          </details>
        </article>
        <div v-else class="session-empty">
          <Leaf :size="20" />
          <p>
            {{
              store.selectedSlot.is_sensory_break
                ? "Take a quiet break between sessions. Your map can help you find a little breathing room."
                : "There is no session recommended for this time. Browse the map or meet someone nearby."
            }}
          </p>
        </div>
      </details>
      <button
        class="session-map-button"
        :disabled="!store.activeTalk && !store.haven"
        @click="store.showSessionOnMap()"
      >
        <MapPin :size="15" /> View on map <ArrowUpRight :size="14" />
      </button>
    </div>
    <SensoryHavenBanner />
  </section>
</template>
