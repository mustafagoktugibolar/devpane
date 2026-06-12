<script setup lang="ts">
import { ref } from 'vue';
import type { WorkspacePaneState } from '../types';

const props = defineProps<{
  name: string;
  path: string | null;
  suggestedPath: string;
  panes: WorkspacePaneState[];
  commandLogs: Record<string, string[]>;
  error: string | null;
}>();

const emit = defineEmits<{
  cancel: [];
  save: [payload: { name: string; path: string; commands: Record<string, string> }];
}>();

const name = ref(props.name);
const path = ref(props.path ?? props.suggestedPath);
const commands = ref<Record<string, string>>(
  Object.fromEntries(props.panes.map(pane => [pane.id, pane.command])),
);

function sessionLog(paneId: string): string[] {
  return props.commandLogs[paneId] ?? [];
}

function useCapturedCommands(paneId: string) {
  commands.value[paneId] = sessionLog(paneId).join('\n');
}

function submit() {
  emit('save', {
    name: name.value,
    path: path.value,
    commands: { ...commands.value },
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

      <div class="save-panes">
        <span class="save-panes-title">Startup commands (run after the terminal opens)</span>
        <div v-for="pane in panes" :key="pane.id" class="save-pane">
          <div class="save-pane-head">
            <span class="save-pane-name">{{ pane.name }}</span>
            <button
              v-if="sessionLog(pane.id).length > 0"
              type="button"
              class="save-pane-history"
              :title="sessionLog(pane.id).join('\n')"
              @click="useCapturedCommands(pane.id)"
            >
              Suggest captured commands ({{ sessionLog(pane.id).length }})
            </button>
          </div>
          <textarea
            v-model="commands[pane.id]"
            class="path-input command-input"
            rows="2"
            placeholder="One command per line, e.g. cd web&#10;npm run dev"
            spellcheck="false"
          ></textarea>
        </div>
      </div>

      <div v-if="error" class="error-message">{{ error }}</div>
      <div class="modal-actions">
        <button class="secondary-btn" type="button" @click="emit('cancel')">Cancel</button>
        <button class="primary-btn" type="submit">Save</button>
      </div>
    </form>
  </div>
</template>
