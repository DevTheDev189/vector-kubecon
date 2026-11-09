<script setup lang="ts">
import { Zap } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <section class="hallway-section">
    <div class="section-heading">
      <div>
        <span class="eyebrow">THE HALLWAY TRACK</span>
        <h2>Your kind of people.</h2>
      </div>
      <span class="live-label"><span class="status-dot" /> NEARBY</span>
    </div>
    <TransitionGroup name="peer" tag="div" class="peers"
      ><article
        v-for="peer in store.visiblePeers"
        :key="peer.username"
        class="peer-card"
      >
        <img class="avatar" :src="peer.avatar_url" alt="" />
        <div class="peer-info">
          <strong>{{ peer.display_name }}</strong
          ><span
            >{{ (peer.similarity_pct * 100).toFixed(1) }}% DNA match ·
            {{
              peer.shared_tags.slice(0, 2).join(" + ") || peer.persona_title
            }}</span
          ><small>{{ peer.shared_context_reason }}</small>
        </div>
        <button
          class="icon-button"
          :disabled="store.busy"
          :aria-label="'Handshake with ' + peer.display_name"
          @click="store.openBadge(peer.username)"
        >
          <Zap :size="16" />
        </button></article
    ></TransitionGroup>
    <p v-if="!store.visiblePeers.length" class="muted empty-inline">
      No nearby matches yet. Your radar is listening.
    </p>
  </section>
</template>
