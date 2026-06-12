<script setup lang="ts">
import { computed } from 'vue';
import WorkspaceNodeView from './WorkspaceNodeView.vue';
import { paneMapById } from '../layout';
import type { WorkspaceState } from '../types';

const props = defineProps<{
  workspace: WorkspaceState;
}>();

const emit = defineEmits<{
  'remove-terminal': [paneId: string];
  'focus-pane': [paneId: string];
  'resize-split': [payload: { path: number[]; sizes: number[] }];
}>();

const panesById = computed(() => paneMapById(props.workspace.panes));
</script>

<template>
  <section class="workspace-shell">
    <WorkspaceNodeView
      :node="workspace.layout"
      :panes-by-id="panesById"
      :root="workspace.root"
      :active-pane-id="workspace.activePaneId"
      :scrollback="workspace.scrollback"
      :path="[]"
      @focus-pane="paneId => emit('focus-pane', paneId)"
      @close-pane="paneId => emit('remove-terminal', paneId)"
      @resize-split="payload => emit('resize-split', payload)"
    />
  </section>
</template>
