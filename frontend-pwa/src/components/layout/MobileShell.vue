<script setup lang="ts">
import TopTelemetryPill from "./TopTelemetryPill.vue";
import GithubLoginHero from "../onboarding/GithubLoginHero.vue";
import DnaExtractionTerminal from "../onboarding/DnaExtractionTerminal.vue";
import SaltPalaceSpatialMap from "../map/SaltPalaceSpatialMap.vue";
import UnifiedLiveSheet from "../feed/UnifiedLiveSheet.vue";
import HallwayProximityStrip from "../hallway/HallwayProximityStrip.vue";
import ConnectionVaultDrawer from "../hallway/ConnectionVaultDrawer.vue";
import QrBadgeModal from "../hallway/QrBadgeModal.vue";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <section class="mobile-shell" aria-label="Attendee live radar">
    <div class="phone-status">
      <span>VECTOR / LIVE RADAR</span><span>▮▮▮ ◉</span>
    </div>
    <TopTelemetryPill />
    <div v-if="store.error" class="error-banner" role="alert">
      {{ store.error }}
      <button @click="store.error = ''" aria-label="Dismiss error">×</button>
    </div>
    <GithubLoginHero /><DnaExtractionTerminal /><SaltPalaceSpatialMap />
    <div v-if="store.loading" class="loading-state" role="status">
      <span class="status-dot" /> Connecting your conference…
    </div>
    <div v-else-if="!store.profile" class="empty-state">
      <p>Your radar is waiting for the backend.</p>
      <button @click="store.refresh()">Reconnect</button>
    </div>
    <template v-else><UnifiedLiveSheet /><HallwayProximityStrip /></template>
    <ConnectionVaultDrawer /><QrBadgeModal />
  </section>
</template>
