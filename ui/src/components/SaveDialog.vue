<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps<{
  name: string;
  path: string | null;
  suggestedPath: string;
  error: string | null;
}>();

const emit = defineEmits<{
  cancel: [];
  save: [payload: { name: string; path: string }];
}>();

const name = ref(props.name);
const path = ref(props.path ?? props.suggestedPath);

function submit() {
  emit('save', {
    name: name.value,
    path: path.value,
  });
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('cancel')">
    <form class="modal" @submit.prevent="submit">
      <h2>Save workspace</h2>
      <label>
        <span>Workspace name</span>
        <input v-model="name" class="path-input" />
      </label>
      <label>
        <span>File path</span>
        <input v-model="path" class="path-input" />
      </label>
      <div v-if="error" class="error-message">{{ error }}</div>
      <div class="modal-actions">
        <button class="secondary-btn" type="button" @click="emit('cancel')">Cancel</button>
        <button class="primary-btn" type="submit">Save</button>
      </div>
    </form>
  </div>
</template>
