<script setup lang="ts">
import { ref, watch } from "vue";
import QRCode from "qrcode";
import DialogPanel from "../layout/DialogPanel.vue";
import { useVectorStore } from "../../stores/vectorStore";
const store = useVectorStore(),
  image = ref(""),
  username = ref(""),
  pin = ref("");
watch(
  () => store.isQrModalOpen,
  (open) => {
    if (open) {
      username.value = store.qrPeer;
      const params = new URLSearchParams(location.search);
      pin.value =
        params.get("peer") === store.qrPeer ? params.get("pin") || "" : "";
    }
  },
);
watch(
  () => store.badge,
  async (value) => {
    image.value = "";
    if (value) {
      try {
        image.value = await QRCode.toDataURL(
          `${location.origin}/?peer=${encodeURIComponent(value.badge.username)}&pin=${encodeURIComponent(value.pin_code)}`,
          { width: 320, margin: 3, errorCorrectionLevel: "M" },
        );
      } catch {
        store.error = "Unable to generate badge. Please retry.";
      }
    }
  },
);
</script>
<template>
  <DialogPanel
    v-if="store.isQrModalOpen"
    title="Make the hallway count."
    @close="store.isQrModalOpen = false"
    ><p class="muted">
      Share your badge, or connect with someone you just met.
    </p>
    <div class="qr-badge">
      <img
        v-if="image"
        :src="image"
        alt="Scan to open my VECTOR handshake badge"
        width="260"
        height="260"
      />
      <p v-else role="status">
        {{
          store.busy
            ? "Preparing your badge…"
            : "Badge unavailable. Close and retry."
        }}
      </p>
      <strong
        >@{{ store.badge?.badge.username || store.profile?.username }}</strong
      ><span class="badge-pin">{{
        store.badge?.pin_code ? "#" + store.badge.pin_code : "••••"
      }}</span>
    </div>
    <form @submit.prevent="store.connectPeer(username, pin)">
      <label
        >Their GitHub handle<input
          v-model="username"
          required
          pattern="[A-Za-z0-9-]+"
          placeholder="someone-you-met"
          autocomplete="off" /></label
      ><label
        >Their PIN <span class="muted">(optional)</span
        ><input
          v-model="pin"
          inputmode="numeric"
          pattern="[0-9]{4}"
          maxlength="4"
          placeholder="4 digits"
      /></label>
      <p v-if="store.error" role="alert" class="error-text">
        {{ store.error }}
      </p>
      <button class="primary" :disabled="store.busy">
        {{
          store.busy
            ? "Preparing your follow-up…"
            : "⚡ Connect & draft follow-up"
        }}
      </button>
    </form>
    <p class="micro muted">
      Your meeting context becomes a personal LinkedIn and email draft.
    </p></DialogPanel
  >
</template>
