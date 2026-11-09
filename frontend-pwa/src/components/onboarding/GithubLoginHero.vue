<script setup lang="ts">
import { ref } from "vue";
import { Github, ChevronDown, ArrowUpRight } from "lucide-vue-next";
import { judgeHandles, useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore(),
  expanded = ref(false),
  username = ref(""),
  linkedin = ref("");
async function sync() {
  await store.sync(
    username.value.trim().replace(/^@/, ""),
    linkedin.value,
    false,
  );
  if (!store.error) expanded.value = false;
}
</script>
<template>
  <div class="identity-switcher">
    <button
      class="identity-button"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <Github :size="15" /><span>{{
        store.profile
          ? "@" + store.profile.username
          : "Find your conference DNA"
      }}</span
      ><ChevronDown :size="14" /></button
    ><span class="micro muted">GITHUB CONNECTED DNA</span>
  </div>
  <div
    v-if="expanded || (!store.profile && !store.loading)"
    class="onboarding-panel"
  >
    <a class="button" href="/api/auth/github"
      ><Github :size="16" /> Continue with GitHub <ArrowUpRight :size="14"
    /></a>
    <p class="eyebrow">OR EXPLORE A JUDGE PROFILE</p>
    <div class="preset-list">
      <button
        v-for="handle in judgeHandles"
        :key="handle"
        :disabled="store.busy"
        @click="store.sync(handle)"
      >
        @{{ handle }}
      </button>
    </div>
    <form @submit.prevent="sync">
      <label
        >GitHub handle<input
          v-model="username"
          required
          pattern="[A-Za-z0-9-]+"
          placeholder="your-handle"
          autocomplete="username" /></label
      ><label
        >LinkedIn URL <span class="muted">(optional)</span
        ><input
          v-model="linkedin"
          type="url"
          placeholder="https://linkedin.com/in/…" /></label
      ><button class="primary" :disabled="store.busy">
        {{ store.busy ? "Syncing…" : "Build my radar ↗" }}
      </button>
    </form>
  </div>
</template>
