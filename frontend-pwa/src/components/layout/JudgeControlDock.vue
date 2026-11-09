<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import QRCode from "qrcode";
import {
  Activity,
  ArrowUpRight,
  Zap,
  RotateCcw,
  AudioLines,
  Radio,
  Search,
  Server,
  Smartphone,
} from "lucide-vue-next";
import { requestTelemetry } from "../../api/client";
import { judgeHandles, useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore(),
  tab = ref("telemetry"),
  phoneQr = ref("");
const tabs = [
  { id: "telemetry", label: "Telemetry", icon: Radio },
  { id: "elastic", label: "Elastic RRF", icon: Search },
  { id: "infra", label: "Infrastructure", icon: Server },
  { id: "phone", label: "Try on phone", icon: Smartphone },
];
const json = computed(() => JSON.stringify(store.lastElasticDsl, null, 2));
onMounted(async () => {
  try {
    phoneQr.value = await QRCode.toDataURL("https://vector.winslow.plus", {
      width: 240,
      margin: 3,
    });
  } catch {
    store.error = "Unable to generate the phone link QR.";
  }
});
</script>
<template>
  <aside class="judge-dock">
    <div class="dock-intro">
      <span class="eyebrow accent">THE CONFERENCE, REORCHESTRATED.</span>
      <h1>
        Find your session.<br />Find your people.<br /><span
          >Keep your flow.</span
        >
      </h1>
      <p>
        A live spatial radar that moves with the room.<br />Built for everything
        that happens between the talks.
      </p>
    </div>
    <section class="control-panel">
      <div class="section-heading">
        <span class="eyebrow"><Activity :size="14" /> LIVE CONTROL ROOM</span
        ><span class="live-label"
          ><span
            class="status-dot"
            :class="{ disconnected: !store.sseConnected }"
          />
          {{ store.sseConnected ? "STREAMING" : "CONNECTING" }}</span
        >
      </div>
      <div class="chaos-controls">
        <button
          class="surge-control"
          :disabled="store.busy || !store.profile"
          @click="store.triggerCrowdSurge()"
        >
          <Zap :size="20" />
          <div>
            <strong>Trigger crowd surge</strong
            ><small>Ballroom B → find the next best room</small>
          </div>
          <ArrowUpRight :size="18" />
        </button>
        <div class="button-row">
          <button
            :disabled="store.busy || !store.profile"
            @click="store.resetSurge()"
          >
            <RotateCcw :size="14" /> Reset surge</button
          ><button
            :aria-pressed="store.quietMode"
            :disabled="store.busy || !store.profile"
            @click="store.toggleQuiet()"
          >
            <AudioLines :size="14" />
            {{ store.quietMode ? "Quiet mode on" : "Try quiet mode" }}
          </button>
        </div>
        <button
          class="text-button noise-control"
          :disabled="store.busy || !store.currentRoom"
          @click="store.sampleNoise()"
        >
          ↗ Submit current room noise reading
        </button>
        <p class="micro muted">
          Reuses the latest reading; no microphone recording.
        </p>
      </div>
      <div class="judge-presets">
        <span class="eyebrow muted">EXPLORE A DIFFERENT BUILDER DNA</span>
        <div class="preset-list">
          <button
            v-for="handle in judgeHandles"
            :key="handle"
            :class="{ active: store.profile?.username === handle }"
            :disabled="store.busy"
            @click="store.sync(handle)"
          >
            @{{ handle }}
          </button>
        </div>
      </div>
    </section>
    <section class="proof-panel">
      <div class="proof-tabs" role="tablist" aria-label="Backend proof panels">
        <button
          v-for="item in tabs"
          :id="'tab-' + item.id"
          :key="item.id"
          role="tab"
          :aria-selected="tab === item.id"
          :aria-controls="'panel-' + item.id"
          @click="tab = item.id"
        >
          <component :is="item.icon" :size="14" />{{ item.label }}
        </button>
      </div>
      <div
        :id="'panel-' + tab"
        role="tabpanel"
        :aria-labelledby="'tab-' + tab"
        class="proof-content"
      >
        <template v-if="tab === 'telemetry'"
          ><div class="metric-row">
            <div>
              <span class="eyebrow muted">LAST REQUEST</span
              ><strong
                >{{ requestTelemetry.lastRequestMs }}<small>ms</small></strong
              >
            </div>
            <div>
              <span class="eyebrow muted">ROOMS ON RADAR</span
              ><strong>{{
                store.rooms.length.toString().padStart(2, "0")
              }}</strong>
            </div>
            <div>
              <span class="eyebrow muted">LIVE EVENTS</span
              ><strong>{{
                store.sseEventLog.length.toString().padStart(2, "0")
              }}</strong>
            </div>
          </div>
          <div class="trace-box">
            <span class="eyebrow muted">REQUEST TRACE ID</span
            ><code>{{
              store.lastTraceId || "Waiting for first request…"
            }}</code>
          </div>
          <div class="room-gauges">
            <div v-for="room in store.rooms.slice(0, 4)" :key="room.room_id">
              <span>{{ room.room_name }}</span>
              <div class="progress-track">
                <i
                  :style="{
                    width: Math.min(100, room.occupancy_ratio * 100) + '%',
                  }"
                />
              </div>
              <code>{{ Math.round(room.occupancy_ratio * 100) }}%</code>
            </div>
          </div>
          <div class="event-feed">
            <div class="section-heading">
              <span class="eyebrow muted">SSE EVENT STREAM</span
              ><span class="micro">LATEST 25</span>
            </div>
            <p v-if="!store.sseEventLog.length" class="muted">
              Listening. Trigger a surge to see the room react.
            </p>
            <p v-for="(event, index) in store.sseEventLog" :key="index">
              <time>{{
                new Date(event.data.timestamp_utc).toLocaleTimeString()
              }}</time
              ><span>{{ event.event_type }}</span>
            </p>
          </div>
          <p class="micro muted">
            Backend trace IDs and room readings. Dash0 export depends on server
            configuration.
          </p></template
        >
        <template v-else-if="tab === 'elastic'"
          ><div class="section-heading">
            <span class="eyebrow accent">LIVE QUERY / ELASTICSEARCH</span
            ><span class="micro muted">1536 DIMENSIONS</span>
          </div>
          <p class="muted">
            The exact query returned by your latest recommendation request.
          </p>
          <pre class="code-view">{{ json || "No query received yet." }}</pre>
        </template>
        <template v-else-if="tab === 'infra'"
          ><div class="section-heading">
            <span class="eyebrow accent">SERVICE STATUS</span
            ><button class="text-button" @click="store.checkHealth()">
              Check again ↻
            </button>
          </div>
          <h3 class="service-health">
            API
            {{
              store.health === "ok"
                ? "reachable"
                : store.health === "unavailable"
                  ? "unavailable"
                  : "not checked"
            }}
          </h3>
          <p class="muted">
            Pod health and replica counts are not exposed by the current API.
          </p>
          <div class="incident-summary">
            <span class="eyebrow muted">VERIFIED AUDIT EXERCISE</span>
            <div class="performance-comparison">
              <span><s>173.8</s><small>ms / 15 requests</small></span
              ><span>→</span
              ><span class="accent">16.5<small>ms / 1 batch</small></span>
            </div>
            <p>
              Local median across seven runs. Deliberate sequential baseline
              replaced with one Elasticsearch batch.
            </p>
            <a
              href="https://github.com/DevTheDev189/vector-kubecon/blob/main/docs/SRE_INCIDENT_REPORT.md"
              target="_blank"
              rel="noopener noreferrer"
              class="text-button"
              >Read measured incident report ↗</a
            >
          </div></template
        >
        <template v-else
          ><div class="phone-link">
            <img
              v-if="phoneQr"
              :src="phoneQr"
              width="210"
              height="210"
              alt="Scan to visit vector.winslow.plus"
            />
            <h3>Your phone. Your radar.</h3>
            <a
              href="https://vector.winslow.plus"
              target="_blank"
              rel="noopener noreferrer"
              >vector.winslow.plus ↗</a
            >
            <p class="muted">
              Public deployment link. Availability depends on production
              deployment.
            </p>
          </div></template
        >
      </div>
    </section>
    <p class="dock-footnote">
      <span class="accent">✳</span> Rust / Elasticsearch / OpenTelemetry /
      OpenAI
    </p>
  </aside>
</template>
