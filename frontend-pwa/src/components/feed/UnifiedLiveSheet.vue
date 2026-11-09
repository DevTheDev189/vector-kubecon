<script setup lang="ts">
import { Bookmark, ArrowUpRight, AudioLines } from "lucide-vue-next";
import { localTime, useVectorStore } from "../../stores/vectorStore";
import SensoryHavenBanner from "./SensoryHavenBanner.vue";
import SurgeRerouteAlert from "./SurgeRerouteAlert.vue";
const store = useVectorStore();
</script>
<template>
  <section class="live-sheet">
    <div class="sheet-handle" />
    <div v-if="store.profile" class="dna-bar">
      <img :src="store.profile.avatar_url" alt="" class="avatar" />
      <div>
        <span class="eyebrow">YOUR BUILDER DNA</span>
        <h2>{{ store.profile.persona_title }}</h2>
      </div>
      <span class="dna-mark">✳</span>
    </div>
    <div class="tag-list">
      <span v-for="tag in store.profile?.dna_tags.slice(0, 4)" :key="tag">{{
        tag
      }}</span>
    </div>
    <details class="icebreaker">
      <summary>Your hallway opener ↗</summary>
      <p>{{ store.profile?.hallway_icebreaker }}</p>
    </details>
    <div class="section-heading schedule-heading">
      <span class="eyebrow">YOUR DAY, RECOMPUTED.</span
      ><span class="micro muted">MOUNTAIN TIME</span>
    </div>
    <div class="slot-scrubber" aria-label="Conference time slots">
      <button
        v-for="(slot, index) in store.slots"
        :key="slot.start_time"
        :class="{ active: store.selectedSlotIndex === index }"
        :aria-pressed="store.selectedSlotIndex === index"
        @click="
          store.selectedSlotIndex = index;
          store.selectedRoomId = '';
        "
      >
        {{ localTime(slot.start_time)
        }}<span v-if="index === 2" class="micro">DEMO SLOT</span>
      </button>
    </div>
    <SensoryHavenBanner /><SurgeRerouteAlert />
    <Transition name="card" mode="out-in"
      ><article
        v-if="store.activeTalk"
        :key="store.activeTalk.talk_id"
        class="talk-card"
      >
        <div class="section-heading">
          <span class="eyebrow accent">{{
            store.activeTalk.is_rerouted_alternative
              ? "↗ YOUR OPEN ALTERNATIVE"
              : "✳ RECOMMENDED FOR YOU"
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
            >{{ store.activeTalk.room_name }} <ArrowUpRight :size="13" /></span
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
          <summary>Why this session matches your DNA</summary>
          <p v-for="reason in store.activeTalk.match_reasons" :key="reason">
            {{ reason }}
          </p>
          <p class="mono">
            RRF {{ store.activeTalk.rrf_score.toFixed(4) }} · Vector similarity
            {{ (store.activeTalk.vector_similarity * 100).toFixed(1) }}%
          </p>
        </details>
      </article>
      <div v-else class="empty-state">
        <h3>
          {{
            store.selectedSlot?.is_sensory_break
              ? "Your quiet break."
              : "No session matches this slot."
          }}
        </h3>
        <p>
          {{
            store.selectedSlot?.is_sensory_break
              ? "Take a moment to recharge in a quiet haven."
              : "Try another time or adjust your sensory preferences."
          }}
        </p>
      </div></Transition
    >
  </section>
</template>
