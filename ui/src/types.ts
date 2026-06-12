export interface RecentSession {
  path: string;
  name: string;
  last_opened: number;
}

export interface PaneSummary {
  id: string;
  name: string;
  auto_start: boolean;
  command: string | null;
  cwd: string;
  shell: string;
}

export type SplitDirection = 'horizontal' | 'vertical';

export interface WorkspacePaneState {
  id: string;
  name: string;
  command: string;
  cwd: string | null;
  shell: string | null;
  autoStart: boolean;
}

export interface WorkspaceLayoutPane {
  kind: 'pane';
  pane: string;
  size: number | null;
}

export interface WorkspaceLayoutSplit {
  kind: 'split';
  direction: SplitDirection;
  size: number | null;
  children: WorkspaceLayoutNode[];
}

export type WorkspaceLayoutNode = WorkspaceLayoutPane | WorkspaceLayoutSplit;

export interface WorkspaceSummary {
  name: string;
  root: string;
  layout: WorkspaceLayoutNode;
  panes: PaneSummary[];
  scrollback: number;
}

export interface WorkspaceState {
  name: string;
  path: string | null;
  root: string | null;
  layout: WorkspaceLayoutNode;
  panes: WorkspacePaneState[];
  activePaneId: string | null;
  scrollback: number;
  dirty: boolean;
}

export interface TerminalOutput {
  pane_id: string;
  data: string;
}

export interface TerminalExit {
  pane_id: string;
}

export type ResizeDirection =
  | 'East'
  | 'North'
  | 'NorthEast'
  | 'NorthWest'
  | 'South'
  | 'SouthEast'
  | 'SouthWest'
  | 'West';
