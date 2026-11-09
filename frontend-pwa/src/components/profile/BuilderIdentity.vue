<script setup lang="ts">
import { LogOut, RefreshCw, ChevronDown } from "lucide-vue-next";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <section
    v-if="store.profile"
    class="builder-identity"
    aria-labelledby="you-heading"
  >
    <div class="section-heading">
      <h2 id="you-heading">You</h2>
      <span class="micro muted">YOUR BUILDER DNA</span>
    </div>
    <div class="builder-summary">
      <img class="avatar" :src="store.profile.avatar_url" alt="" />
      <div>
        <strong>{{ store.profile.display_name }}</strong>
        <p>{{ store.profile.persona_title }}</p>
      </div>
    </div>
    <div class="tag-list">
      <span v-for="tag in store.profile.dna_tags.slice(0, 4)" :key="tag">{{
        tag
      }}</span>
    </div>
    <details class="icebreaker">
      <summary>Your hallway opener <ChevronDown :size="13" /></summary>
      <p>{{ store.profile.hallway_icebreaker }}</p>
    </details>
    <details class="account-options">
      <summary>Account settings <ChevronDown :size="13" /></summary>
      <div class="signed-in-account">
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
          <RefreshCw :size="15" /></button
        ><button
          class="icon-button"
          :disabled="store.busy"
          aria-label="Sign out"
          @click="store.logout()"
        >
          <LogOut :size="15" />
        </button>
      </div>
    </details>
  </section>
</template>
