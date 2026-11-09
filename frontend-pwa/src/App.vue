<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import MobileShell from "./components/layout/MobileShell.vue";
import JudgeControlDock from "./components/layout/JudgeControlDock.vue";
import { useVectorStore } from "./stores/vectorStore";
const store = useVectorStore();
onMounted(async () => {
  store.connectSseStream();
  void store.checkHealth();
  await store.refresh();
  const params = new URLSearchParams(location.search),
    peer = params.get("peer");
  if (peer && /^[A-Za-z0-9-]+$/.test(peer) && store.profile)
    await store.openBadge(peer);
});
onUnmounted(store.disconnect);
</script>
<template>
  <main class="workspace" :class="{ quiet: store.quietMode }">
    <header class="brand-header">
      <a href="/" class="wordmark"
        ><span class="brand-icon">∨</span> VECTOR<span class="wordmark-dot"
          >®</span
        ></a
      ><span class="eyebrow"
        >KUBECON NA ’26 <span class="muted">/ SALT LAKE CITY</span></span
      ><span class="live-label"><i class="status-dot" /> CONFERENCE OS</span>
    </header>
    <div class="workspace-grid"><MobileShell /><JudgeControlDock /></div>
    <footer class="workspace-footer">
      <span>LESS FRICTION. MORE CONNECTION.</span
      ><span>BUILT FOR THE HALLWAY TRACK ↗</span>
    </footer>
  </main>
</template>
