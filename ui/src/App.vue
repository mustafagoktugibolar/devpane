<script setup lang="ts">
import { onMounted, ref } from 'vue';
import Titlebar from './components/Titlebar.vue';
import SessionPicker from './components/SessionPicker.vue';
import TerminalWorkspace from './components/TerminalWorkspace.vue';
import SaveDialog from './components/SaveDialog.vue';
import DeleteWorkspaceDialog from './components/DeleteWorkspaceDialog.vue';
import {
  addRecentSession,
  deleteWorkspace,
  listRecentSessions,
  loadWorkspace,
  saveWorkspace,
  suggestWorkspacePath,
} from './api/workspaces';
import { setWindowTitle } from './api/window';
import {
  createPaneNode,
  firstPaneId,
  insertPaneIntoLayout,
  normalizePaneSizes,
  removePaneFromLayout,
  updateSplitSizes,
} from './layout';
import * as sessions_api from './terminalSessions';
import type {
  RecentSession,
  SplitDirection,
  WorkspacePaneState,
  WorkspaceState,
  WorkspaceSummary,
} from './types';

type ViewMode = 'picker' | 'workspace';

const mode = ref<ViewMode>('picker');
const sessions = ref<RecentSession[]>([]);
const pickerError = ref<string | null>(null);
const workspace = ref<WorkspaceState | null>(null);
const showSave = ref(false);
const saveError = ref<string | null>(null);
const savePathSuggestion = ref('');
const deleteError = ref<string | null>(null);
const pendingDelete = ref<RecentSession | null>(null);

const DEFAULT_SCROLLBACK = 1000;

onMounted(() => {
  void refreshSessions();
  void setWindowTitle();
});

async function refreshSessions() {
  sessions.value = await listRecentSessions();
}

function makePane(index: number, shell?: string | null): WorkspacePaneState {
  return {
    id: `pane-${index}`,
    name: `Terminal ${index}`,
    command: '',
    cwd: null,
    shell: shell ?? null,
    autoStart: true,
  };
}

function nextPaneIndex(): number {
  const panes = workspace.value?.panes ?? [];
  let index = panes.length + 1;

  while (panes.some(pane => pane.id === `pane-${index}`)) {
    index += 1;
  }

  return index;
}

function makeInitialWorkspace(): WorkspaceState {
  const pane = makePane(1);

  return {
    name: 'Untitled Workspace',
    path: null,
    root: null,
    layout: createPaneNode(pane.id),
    panes: [pane],
    activePaneId: pane.id,
    scrollback: DEFAULT_SCROLLBACK,
    dirty: true,
  };
}

function panesFromSummary(summary: WorkspaceSummary): WorkspacePaneState[] {
  return summary.panes.map((pane, index) => ({
    id: pane.id,
    name: pane.name || `Terminal ${index + 1}`,
    command: pane.command ?? '',
    cwd: pane.cwd,
    shell: pane.shell,
    autoStart: pane.auto_start,
  }));
}

function startNewSession() {
  void sessions_api.stopAll();
  sessions_api.setScrollback(DEFAULT_SCROLLBACK);
  workspace.value = makeInitialWorkspace();
  pickerError.value = null;
  mode.value = 'workspace';
  void setWindowTitle(workspace.value.name);
}

async function openSession(path: string) {
  try {
    const loaded = await loadWorkspace(path);
    const panes = panesFromSummary(loaded);

    void sessions_api.stopAll();
    sessions_api.setScrollback(loaded.scrollback);
    workspace.value = {
      name: loaded.name,
      path,
      root: loaded.root,
      layout: loaded.layout,
      panes: panes.length > 0 ? panes : [makePane(1)],
      activePaneId: firstPaneId(loaded.layout) ?? panes[0]?.id ?? null,
      scrollback: loaded.scrollback,
      dirty: false,
    };
    await addRecentSession(path, loaded.name);
    await refreshSessions();
    pickerError.value = null;
    mode.value = 'workspace';
    void setWindowTitle(loaded.name);
  } catch (error) {
    pickerError.value = error instanceof Error ? error.message : String(error);
    mode.value = 'picker';
  }
}

function requestDeleteWorkspace(path: string) {
  const session = sessions.value.find(item => item.path === path);
  pendingDelete.value = session ?? { path, name: path, last_opened: 0 };
  deleteError.value = null;
}

async function confirmDeleteWorkspace() {
  if (!pendingDelete.value) return;

  const path = pendingDelete.value.path;
  try {
    await deleteWorkspace(path);
    await refreshSessions();
    pendingDelete.value = null;
    deleteError.value = null;
    pickerError.value = null;
  } catch (error) {
    deleteError.value = error instanceof Error ? error.message : String(error);
  }
}

function showSessions() {
  void sessions_api.stopAll();
  workspace.value = null;
  mode.value = 'picker';
  void refreshSessions();
  void setWindowTitle();
}

function focusPane(paneId: string) {
  if (!workspace.value) return;
  workspace.value = {
    ...workspace.value,
    activePaneId: paneId,
  };
}

function splitActivePane(direction: SplitDirection, shell?: string | null) {
  if (!workspace.value) return;

  const activePaneId = workspace.value.activePaneId ?? firstPaneId(workspace.value.layout);
  if (!activePaneId) return;

  const nextPane = makePane(nextPaneIndex(), shell);

  const nextLayout = insertPaneIntoLayout(workspace.value.layout, activePaneId, direction, nextPane.id);
  if (nextLayout === workspace.value.layout) return;

  workspace.value = {
    ...workspace.value,
    layout: normalizePaneSizes(nextLayout),
    panes: [...workspace.value.panes, nextPane],
    activePaneId: nextPane.id,
    dirty: true,
  };
}

function closeActiveTerminal() {
  const activePaneId = workspace.value?.activePaneId;
  if (activePaneId) {
    removeTerminal(activePaneId);
  }
}

function removeTerminal(paneId: string) {
  if (!workspace.value) return;

  void sessions_api.stop(paneId);

  const nextLayout = removePaneFromLayout(workspace.value.layout, paneId);
  const nextPanes = workspace.value.panes.filter(pane => pane.id !== paneId);

  if (!nextLayout) {
    workspace.value = makeInitialWorkspace();
    workspace.value.dirty = true;
    return;
  }

  const activePaneId = workspace.value.activePaneId === paneId ? firstPaneId(nextLayout) : workspace.value.activePaneId;

  if (nextPanes.length === 0) {
    const pane = makePane(1);
    workspace.value = {
      ...workspace.value,
      layout: createPaneNode(pane.id),
      panes: [pane],
      activePaneId: pane.id,
      dirty: true,
    };
    return;
  }

  workspace.value = {
    ...workspace.value,
    layout: normalizePaneSizes(nextLayout),
    panes: nextPanes,
    activePaneId,
    dirty: true,
  };
}

function resizeSplit(payload: { path: number[]; sizes: number[] }) {
  if (!workspace.value) return;
  workspace.value = {
    ...workspace.value,
    layout: updateSplitSizes(workspace.value.layout, payload.path, payload.sizes),
    dirty: true,
  };
}

const saveCommandLogs = ref<Record<string, string[]>>({});

async function requestSave() {
  if (!workspace.value) return;
  saveError.value = null;
  saveCommandLogs.value = Object.fromEntries(
    workspace.value.panes.map(pane => [pane.id, sessions_api.getCommandLog(pane.id)]),
  );
  if (!workspace.value.path) {
    try {
      savePathSuggestion.value = await suggestWorkspacePath(workspace.value.name);
    } catch {
      savePathSuggestion.value = '';
    }
  }
  showSave.value = true;
}

async function confirmSave(payload: { name: string; path: string; commands: Record<string, string> }) {
  if (!workspace.value) return;

  if (!payload.name.trim()) {
    saveError.value = 'Enter a workspace name.';
    return;
  }

  if (!payload.path.trim()) {
    saveError.value = 'Enter a file path.';
    return;
  }

  const panes = workspace.value.panes.map(pane => ({
    ...pane,
    command: (payload.commands[pane.id] ?? pane.command).trim(),
  }));

  try {
    const saved = await saveWorkspace(
      payload.path.trim(),
      payload.name.trim(),
      panes,
      workspace.value.layout,
    );
    await addRecentSession(payload.path.trim(), saved.name);
    workspace.value = {
      ...workspace.value,
      name: saved.name,
      path: payload.path.trim(),
      root: saved.root,
      layout: saved.layout,
      panes: panesFromSummary(saved),
      scrollback: saved.scrollback,
      dirty: false,
    };
    await refreshSessions();
    showSave.value = false;
    saveError.value = null;
    void setWindowTitle(saved.name);
  } catch (error) {
    saveError.value = error instanceof Error ? error.message : String(error);
  }
}
</script>

<template>
  <Titlebar
    :has-workspace="mode === 'workspace'"
    :dirty="workspace?.dirty"
    @new-session="startNewSession"
    @sessions="showSessions"
    @save="requestSave"
    @split-right="(shell) => splitActivePane('vertical', shell)"
    @split-down="(shell) => splitActivePane('horizontal', shell)"
    @close-terminal="closeActiveTerminal"
  />

  <main id="content">
    <SessionPicker
      v-if="mode === 'picker'"
      :sessions="sessions"
      :error="pickerError"
      @new-session="startNewSession"
      @open-session="openSession"
      @delete-session="requestDeleteWorkspace"
    />

    <TerminalWorkspace
      v-else-if="workspace"
      :workspace="workspace"
      @remove-terminal="removeTerminal"
      @focus-pane="focusPane"
      @resize-split="resizeSplit"
    />
  </main>

  <SaveDialog
    v-if="showSave && workspace"
    :name="workspace.name"
    :path="workspace.path"
    :suggested-path="savePathSuggestion"
    :panes="workspace.panes"
    :command-logs="saveCommandLogs"
    :error="saveError"
    @cancel="showSave = false"
    @save="confirmSave"
  />

  <DeleteWorkspaceDialog
    v-if="pendingDelete"
    :name="pendingDelete.name"
    :path="pendingDelete.path"
    :error="deleteError"
    @cancel="pendingDelete = null"
    @confirm="confirmDeleteWorkspace"
  />
</template>
