<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import type { RecentSession } from '../types';

const props = defineProps<{
  sessions: RecentSession[];
  error: string | null;
}>();

const emit = defineEmits<{
  'new-session': [];
  'open-session': [path: string];
  'delete-session': [path: string];
}>();

const pickerEl = ref<HTMLElement | null>(null);
const selectedIndex = ref(0);
const total = computed(() => props.sessions.length + 1);

onMounted(() => {
  pickerEl.value?.focus();
});

function openSelected() {
  if (selectedIndex.value === 0) {
    emit('new-session');
    return;
  }

  const session = props.sessions[selectedIndex.value - 1];
  if (session) emit('open-session', session.path);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault();
    selectedIndex.value = (selectedIndex.value + 1) % total.value;
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    selectedIndex.value = (selectedIndex.value - 1 + total.value) % total.value;
  } else if (event.key === 'Enter') {
    event.preventDefault();
    openSelected();
  } else if (event.key === 'Delete' && selectedIndex.value > 0) {
    const session = props.sessions[selectedIndex.value - 1];
    if (session) {
      event.preventDefault();
      emit('delete-session', session.path);
    }
  }
}
</script>

<template>
  <section ref="pickerEl" class="picker" tabindex="0" @keydown="onKeydown">
    <div class="picker-list">
      <div class="picker-item" :class="{ selected: selectedIndex === 0 }" @click="emit('new-session')">
        <span class="picker-new-icon">+</span>
        <div class="picker-item-info">
          <span class="picker-item-name">New Session</span>
          <span class="picker-item-path">Open a blank terminal workspace</span>
        </div>
      </div>

      <template v-if="sessions.length > 0">
        <div class="picker-divider"></div>
        <div
          v-for="(session, index) in sessions"
          :key="session.path"
          class="picker-item"
          :class="{ selected: selectedIndex === index + 1 }"
          @click="emit('open-session', session.path)"
        >
          <div class="picker-item-info">
            <span class="picker-item-name">{{ session.name }}</span>
            <span class="picker-item-path">{{ session.path }}</span>
          </div>
          <button
            class="picker-delete"
            type="button"
            title="Delete workspace"
            @click.stop="emit('delete-session', session.path)"
          >
            &#x2715;
          </button>
        </div>
      </template>
    </div>

    <div v-if="error" class="error-message picker-error">{{ error }}</div>
    <div class="picker-hint">&#x2191; &#x2193; &nbsp;&middot;&nbsp; &#x23CE; open &nbsp;&middot;&nbsp; Delete remove</div>
  </section>
</template>
