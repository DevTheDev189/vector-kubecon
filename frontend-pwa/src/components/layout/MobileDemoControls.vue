<script setup lang="ts">
import { ref } from "vue";
import { SlidersHorizontal, Zap, RotateCcw } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore(),
  panel = ref<HTMLDetailsElement>();
async function runDemo(action: () => Promise<void>) {
  await action();
  if (!store.error && panel.value) {
    panel.value.open = false;
    document
      .querySelector(".day-calendar")
      ?.scrollIntoView({
        behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches
          ? "auto"
          : "smooth",
        block: "start",
      });
  }
}
</script>
<template>
  <details v-if="store.canControlDemo" ref="panel" class="mobile-demo-controls">
    <summary>
      <SlidersHorizontal :size="14" /> Demo controls<span>ADMIN</span>
    </summary>
    <div>
      <button :disabled="store.busy" @click="runDemo(store.triggerCrowdSurge)">
        <Zap :size="15" /> Trigger surge</button
      ><button :disabled="store.busy" @click="runDemo(store.resetSurge)">
        <RotateCcw :size="15" /> Reset surge
      </button>
    </div>
  </details>
</template>
