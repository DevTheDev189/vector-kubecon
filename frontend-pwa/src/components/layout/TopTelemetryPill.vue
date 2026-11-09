<script setup lang="ts">
import { QrCode, AudioLines, Handshake } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <header class="telemetry-bar attendee-toolbar">
    <div class="telemetry-location">
      <span class="status-dot" :class="{ disconnected: !store.sseConnected }" />
      <div>
        <strong>VECTOR</strong
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
      <AudioLines :size="15" /> Quiet
    </button>
    <button
      class="icon-button"
      aria-label="Open QR badge"
      :disabled="store.busy || !store.profile"
      @click="store.openBadge()"
    >
      <QrCode :size="18" />
    </button>
    <button
      class="icon-button toolbar-vault"
      aria-label="Open saved connections"
      :disabled="!store.profile"
      @click="store.isVaultOpen = true"
    >
      <Handshake :size="18" /><span v-if="store.handshakes.length">{{
        store.handshakes.length
      }}</span>
    </button>
  </header>
</template>
