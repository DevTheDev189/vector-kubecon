<script setup lang="ts">
import { Github, ArrowUpRight, LockKeyhole } from "lucide-vue-next";
import SaltPalaceSpatialMap from "../map/SaltPalaceSpatialMap.vue";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore();
</script>
<template>
  <section class="signin-layout">
    <div class="signin-preview"><SaltPalaceSpatialMap /></div>
    <div class="signin-copy">
      <span class="eyebrow"
        ><LockKeyhole :size="13" /> YOUR CONFERENCE, CONNECTED.</span
      >
      <h1>Find your place.<br />Meet your people.</h1>
      <p>
        One map for the sessions, spaces, and conversations that matter to you.
      </p>
      <a
        v-if="store.oauthConfigured"
        class="button primary"
        href="/api/auth/github"
        ><Github :size="18" /> Sign in with GitHub <ArrowUpRight :size="16"
      /></a>
      <p v-else class="config-note" role="status">
        {{
          store.oauthConfigured === false
            ? "GitHub sign-in is being configured. Please check back shortly."
            : "Unable to reach sign-in services. Please reconnect."
        }}
      </p>
      <p class="cookie-note">
        We use an essential, signed session cookie to keep you signed in. No
        advertising cookies. GitHub verifies your identity; your password stays
        with GitHub.
      </p>
      <p v-if="store.error" class="config-note" role="alert">
        {{ store.error }}
      </p>
      <button
        class="text-button"
        :disabled="store.busy"
        @click="store.initialize()"
      >
        Check connection ↗
      </button>
    </div>
  </section>
</template>
