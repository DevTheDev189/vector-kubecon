<script setup lang="ts">
import MobileDemoControls from "./MobileDemoControls.vue";
import TopTelemetryPill from "./TopTelemetryPill.vue";
import SaltPalaceSpatialMap from "../map/SaltPalaceSpatialMap.vue";
import UnifiedLiveSheet from "../feed/UnifiedLiveSheet.vue";
import BuilderIdentity from "../profile/BuilderIdentity.vue";
import HallwayProximityStrip from "../hallway/HallwayProximityStrip.vue";
import ConnectionVaultDrawer from "../hallway/ConnectionVaultDrawer.vue";
import QrBadgeModal from "../hallway/QrBadgeModal.vue";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <section class="mobile-shell" aria-label="Attendee live radar">
    <TopTelemetryPill />
    <div v-if="store.error" class="error-banner" role="alert">
      {{ store.error
      }}<button @click="store.error = ''" aria-label="Dismiss error">×</button>
    </div>
    <div v-if="store.loading" class="loading-state" role="status">
      <span class="status-dot" /> Connecting your conference…
    </div>
    <div v-else-if="!store.profile" class="empty-state">
      <p>Your day is waiting for the backend.</p>
      <button @click="store.refresh()">Reconnect</button>
    </div>
    <template v-else
      ><UnifiedLiveSheet /><SaltPalaceSpatialMap /><BuilderIdentity /><HallwayProximityStrip /><MobileDemoControls
    /></template>
    <ConnectionVaultDrawer /><QrBadgeModal />
  </section>
</template>
