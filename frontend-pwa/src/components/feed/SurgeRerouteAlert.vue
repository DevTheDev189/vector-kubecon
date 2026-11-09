<script setup lang="ts">
import { Route } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <div v-if="store.activeSurgeAlert" class="surge-banner" role="status">
    <Route :size="20" />
    <div>
      <strong>{{ store.activeSurgeAlert.jammed_room_name }} is full.</strong>
      <p>
        {{ store.activeSurgeAlert.occupancy_current }}/{{
          store.activeSurgeAlert.occupancy_max
        }}
        · {{ store.activeSurgeAlert.decibel_level }} dB
      </p>
      <span>{{
        store.activeSurgeAlert.recommended_alternative
          ? "Your recommended day now includes an open alternative."
          : "No open alternative meets your preferences yet."
      }}</span>
    </div>
  </div>
</template>
