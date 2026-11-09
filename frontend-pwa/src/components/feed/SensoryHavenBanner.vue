<script setup lang="ts">
import { Leaf } from "lucide-vue-next";
import { localTime, useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <div v-if="store.quietMode" class="haven-banner">
    <Leaf :size="20" />
    <div>
      <span class="eyebrow">A LITTLE ROOM TO BREATHE</span
      ><template v-if="store.haven"
        ><h3>{{ store.haven.room_name }}</h3>
        <p>
          No active talk{{
            store.haven.next_talk_start
              ? " · Next at " + localTime(store.haven.next_talk_start)
              : ""
          }}
        </p>
        <p>
          {{ Math.round(store.haven.occupancy_ratio * 100) }}% occupied ·
          {{ store.haven.decibel_level }} dB
          <span v-if="store.haven.accessibility.wheelchair_accessible"
            >· Wheelchair access</span
          >
        </p></template
      >
      <p v-else>No verified quiet room is available right now.</p>
    </div>
  </div>
</template>
