<script setup lang="ts">
import { ref, watch } from 'vue';
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
const pathEdited = ref(false);
const commands = ref<Record<string, string>>(
  Object.fromEntries(props.panes.map(pane => [pane.id, pane.command])),
);

function nameToFilename(n: string): string {
  let slug = n
    .toLowerCase()
    .split('')
    .map(c => (/[a-z0-9_-]/).test(c) ? c : '-')
    .join('')
    .replace(/-+/g, '-')
    .replace(/^-+|-+$/g, '');
  return (slug || 'workspace') + '.dpane';
}

watch(name, newName => {
  if (pathEdited.value || props.path) return;
  const sep = path.value.includes('\\') ? '\\' : '/';
  const lastSep = Math.max(path.value.lastIndexOf('\\'), path.value.lastIndexOf('/'));
  const dir = lastSep >= 0 ? path.value.substring(0, lastSep) : '';
  path.value = dir + sep + nameToFilename(newName);
});

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
  <div class="modal-backdrop" @mousedown.self="emit('cancel')">
    <form class="modal" @submit.prevent="submit">
      <h2>Save workspace</h2>
      <label>
        <span>Workspace name</span>
        <input v-model="name" class="path-input" />
      </label>
      <label>
        <span>File path</span>
        <input v-model="path" class="path-input" @input="pathEdited = true" />
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
