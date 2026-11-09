<script setup lang="ts">
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <div
    v-if="store.syncUsername && store.dnaSyncSteps.length"
    class="dna-terminal"
    role="status"
    aria-live="polite"
  >
    <div class="section-heading">
      <span class="eyebrow">DNA SYNC / @{{ store.syncUsername }}</span
      ><span>{{ store.dnaSyncSteps.at(-1)?.progress_pct }}%</span>
    </div>
    <div class="progress-track">
      <i
        :style="{ width: (store.dnaSyncSteps.at(-1)?.progress_pct || 0) + '%' }"
      />
    </div>
    <p v-for="step in store.dnaSyncSteps" :key="step.step">
      <span>›</span> {{ step.step }}<small>{{ step.detail }}</small>
    </p>
    <button
      v-if="!store.busy"
      class="text-button"
      @click="store.syncUsername = ''"
    >
      Close terminal
    </button>
  </div>
</template>
