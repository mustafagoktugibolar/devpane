import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ShellOption, TerminalExit, TerminalOutput } from '../types';

export function listenTerminalOutput(
  paneId: string,
  handler: (output: TerminalOutput) => void,
): Promise<UnlistenFn> {
  return listen<TerminalOutput>(`terminal-output-${paneId}`, event => handler(event.payload));
}

export function listenTerminalExit(
  paneId: string,
  handler: (exit: TerminalExit) => void,
): Promise<UnlistenFn> {
  return listen<TerminalExit>(`terminal-exited-${paneId}`, event => handler(event.payload));
}

export interface StartTerminalOptions {
  paneId: string;
  paneName: string;
  cwd: string | null;
  shell: string | null;
  rows: number;
  cols: number;
}

export function startTerminal(options: StartTerminalOptions): Promise<void> {
  return invoke('start_terminal', {
    request: {
      pane_id: options.paneId,
      pane_name: options.paneName,
      cwd: options.cwd,
      shell: options.shell,
      rows: options.rows,
      cols: options.cols,
    },
  });
}

export function writeTerminal(paneId: string, data: string): Promise<void> {
  return invoke('write_terminal', {
    request: {
      pane_id: paneId,
      data,
    },
  });
}

export function resizeTerminal(paneId: string, rows: number, cols: number): Promise<void> {
  return invoke('resize_terminal', {
    request: {
      pane_id: paneId,
      rows,
      cols,
    },
  });
}

export function stopTerminal(paneId: string): Promise<void> {
  return invoke('stop_terminal', { paneId });
}

export function listShellOptions(): Promise<ShellOption[]> {
  return invoke<ShellOption[]>('list_shell_options');
}
