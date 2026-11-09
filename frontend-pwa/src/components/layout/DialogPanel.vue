<script setup lang="ts">
import { onMounted, ref } from "vue";
import { X } from "lucide-vue-next";
defineProps<{ title: string }>();
const emit = defineEmits<{ close: [] }>(),
  dialog = ref<HTMLDialogElement>();
onMounted(() => dialog.value?.showModal());
</script>
<template>
  <Teleport to="body"
    ><dialog
      ref="dialog"
      class="dialog-panel"
      @cancel.prevent="emit('close')"
      @click="
        (event) => {
          if (event.target === dialog) emit('close');
        }
      "
    >
      <div class="dialog-content">
        <div class="section-heading">
          <h2>{{ title }}</h2>
          <button
            class="icon-button"
            aria-label="Close dialog"
            @click="emit('close')"
          >
            <X :size="20" />
          </button>
        </div>
        <slot />
      </div></dialog
  ></Teleport>
</template>
