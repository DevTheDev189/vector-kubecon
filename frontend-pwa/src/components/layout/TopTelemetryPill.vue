<script setup lang="ts">
import { QrCode, AudioLines } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <div class="telemetry-bar">
    <div class="telemetry-location">
      <span class="status-dot" :class="{ disconnected: !store.sseConnected }" />
      <div>
        <strong>{{
          store.activeSurgeAlert
            ? "SURGE · REROUTING"
            : store.currentRoom?.room_name || "Salt Palace"
        }}</strong
        ><small>{{
          store.sseConnected
            ? "Live radar connected"
            : "Reconnecting live radar…"
        }}</small>
      </div>
    </div>
    <button
      class="icon-button quiet-toggle"
      :aria-pressed="store.quietMode"
      :disabled="store.busy || !store.profile"
      @click="store.toggleQuiet()"
    >
      <AudioLines :size="15" /> Quiet</button
    ><button
      class="icon-button"
      aria-label="Open QR badge"
      :disabled="store.busy || !store.profile"
      @click="store.openBadge()"
    >
      <QrCode :size="18" />
    </button>
  </div>
</template>
