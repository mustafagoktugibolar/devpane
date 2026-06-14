<script setup lang="ts">
defineProps<{
  name: string;
  path: string;
  error: string | null;
}>();

const emit = defineEmits<{
  cancel: [];
  confirm: [];
}>();
</script>

<template>
  <div class="modal-backdrop" @mousedown.self="emit('cancel')">
    <section class="modal" role="dialog" aria-modal="true" aria-labelledby="delete-workspace-title">
      <h2 id="delete-workspace-title">Delete workspace</h2>
      <p class="modal-copy">
        Remove <strong>{{ name }}</strong> from disk and recent sessions.
      </p>
      <p class="modal-path">{{ path }}</p>
      <div v-if="error" class="error-message">{{ error }}</div>
      <div class="modal-actions">
        <button class="secondary-btn" type="button" @click="emit('cancel')">Cancel</button>
        <button class="danger-btn" type="button" @click="emit('confirm')">Delete</button>
      </div>
    </section>
  </div>
</template>
