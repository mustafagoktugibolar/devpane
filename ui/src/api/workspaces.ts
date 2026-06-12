import { invoke } from '@tauri-apps/api/core';
import type {
  RecentSession,
  WorkspaceLayoutNode,
  WorkspacePaneState,
  WorkspaceSummary,
} from '../types';

export function listRecentSessions(): Promise<RecentSession[]> {
  return invoke<RecentSession[]>('list_recent_sessions');
}

export function addRecentSession(path: string, name: string): Promise<void> {
  return invoke('add_recent_session', { path, name });
}

export function suggestWorkspacePath(name: string): Promise<string> {
  return invoke<string>('suggest_workspace_path', { name });
}

export function deleteWorkspace(path: string): Promise<void> {
  return invoke('delete_workspace', {
    request: { path },
  });
}

export function loadWorkspace(path: string): Promise<WorkspaceSummary> {
  return invoke<WorkspaceSummary>('load_workspace', { path });
}

export function saveWorkspace(
  path: string,
  name: string,
  panes: WorkspacePaneState[],
  layout: WorkspaceLayoutNode,
): Promise<WorkspaceSummary> {
  return invoke<WorkspaceSummary>('save_workspace', {
    request: {
      path,
      name,
      layout,
      panes: panes.map(pane => ({
        id: pane.id,
        name: pane.name,
        command: pane.command || null,
        shell: pane.shell || null,
      })),
    },
  });
}
