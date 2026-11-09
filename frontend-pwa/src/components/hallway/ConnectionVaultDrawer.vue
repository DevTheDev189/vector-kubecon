<script setup lang="ts">
import { ref } from "vue";
import { Handshake } from "lucide-vue-next";
import DialogPanel from "../layout/DialogPanel.vue";
import { useVectorStore } from "../../stores/vectorStore";
import type { HandshakeRecord } from "../../types/vector";
const store = useVectorStore(),
  copied = ref(""),
  copyError = ref("");
async function copy(record: HandshakeRecord) {
  try {
    await navigator.clipboard.writeText(record.draft_linkedin_dm);
    copied.value = record.handshake_id;
    copyError.value = "";
  } catch {
    copyError.value = "Clipboard unavailable. Select and copy the draft below.";
  }
}
const mail = (r: HandshakeRecord) =>
  `mailto:?subject=${encodeURIComponent(r.draft_email_subject)}&body=${encodeURIComponent(r.draft_email_body)}`;
</script>
<template>
  <button class="vault-bar" @click="store.isVaultOpen = true">
    <Handshake :size="18" /><span>Saved connections</span
    ><span class="count">{{ store.handshakes.length }}</span
    ><span>↗</span></button
  ><DialogPanel
    v-if="store.isVaultOpen"
    title="Good conversations, kept."
    @close="store.isVaultOpen = false"
    ><p class="muted">Your hallway context. A thoughtful next step.</p>
    <p v-if="!store.handshakes.length" class="empty-state">
      Your first connection starts in the hallway. Tap a nearby attendee to say
      hello.
    </p>
    <p v-if="copyError" role="alert" class="error-text">{{ copyError }}</p>
    <article
      v-for="record in store.handshakes"
      :key="record.handshake_id"
      class="connection-card"
    >
      <div class="dna-bar">
        <img class="avatar" :src="record.peer_avatar_url" alt="" />
        <div>
          <a
            :href="record.peer_github_url"
            target="_blank"
            rel="noopener noreferrer"
            ><strong>{{ record.peer_display_name }} ↗</strong></a
          >
          <p class="muted">{{ record.peer_persona_title }}</p>
        </div>
      </div>
      <p class="micro muted">
        MET AT {{ record.met_room_name }} ·
        {{ new Date(record.created_at).toLocaleDateString() }}
      </p>
      <p>{{ record.ai_context_summary }}</p>
      <details>
        <summary>LinkedIn draft</summary>
        <p class="draft-text">{{ record.draft_linkedin_dm }}</p>
      </details>
      <details>
        <summary>Email draft</summary>
        <strong>{{ record.draft_email_subject }}</strong>
        <p class="draft-text">{{ record.draft_email_body }}</p>
      </details>
      <div class="button-row">
        <button @click="copy(record)">
          {{
            copied === record.handshake_id ? "Copied ✓" : "Copy LinkedIn DM"
          }}</button
        ><a class="button" :href="mail(record)">Open email ↗</a>
      </div>
    </article></DialogPanel
  >
</template>
