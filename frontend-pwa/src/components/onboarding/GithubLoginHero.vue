<script setup lang="ts">
import { LogOut, RefreshCw } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <div v-if="store.profile" class="signed-in-account">
    <img class="avatar" :src="store.profile.avatar_url" alt="" />
    <div>
      <strong>@{{ store.profile.username }}</strong
      ><small>Signed in with GitHub</small>
    </div>
    <button
      class="icon-button"
      :disabled="store.busy"
      aria-label="Refresh my GitHub profile"
      @click="store.sync()"
    >
      <RefreshCw :size="15" />
    </button>
    <button
      class="icon-button"
      :disabled="store.busy"
      aria-label="Sign out"
      @click="store.logout()"
    >
      <LogOut :size="15" />
    </button>
  </div>
</template>
