import type { UnlistenFn } from '@tauri-apps/api/event';
import {
  listenTerminalExit,
  listenTerminalOutput,
  resizeTerminal,
  startTerminal,
  stopTerminal,
  writeTerminal,
} from './api/terminals';

export type SessionStatus = 'idle' | 'running' | 'exited';

export interface SessionStartOptions {
  paneId: string;
  paneName: string;
  cwd: string | null;
  shell: string | null;
  command: string | null;
  rows: number;
  cols: number;
}

interface Session {
  status: SessionStatus;
  buffer: string;
  dataSubscribers: Set<(data: string) => void>;
  statusSubscribers: Set<(status: SessionStatus) => void>;
  unlistenOutput: UnlistenFn | null;
  unlistenExit: UnlistenFn | null;
  lastStart: SessionStartOptions | null;
  commandLog: string[];
  startGeneration: number;
}

// PTY sessions live here, outside the component tree, so layout changes that
// remount panes never restart or kill a running terminal.
const sessions = new Map<string, Session>();

// Rough character budget for the replay buffer, derived from the workspace
// scrollback line count when a session starts.
let bufferLimit = 256_000;

const STARTUP_COMMAND_DELAY_MS = 150;

export function setScrollback(lines: number) {
  bufferLimit = Math.max(64_000, lines * 256);
}

function getOrCreate(paneId: string): Session {
  let session = sessions.get(paneId);
  if (!session) {
    session = {
      status: 'idle',
      buffer: '',
      dataSubscribers: new Set(),
      statusSubscribers: new Set(),
      unlistenOutput: null,
      unlistenExit: null,
      lastStart: null,
      commandLog: [],
      startGeneration: 0,
    };
    sessions.set(paneId, session);
  }
  return session;
}

export function logCommand(paneId: string, command: string): void {
  const session = sessions.get(paneId);
  if (!session) return;
  session.commandLog.push(command);
}

function startupCommandInput(command: string | null): string | null {
  const lines = command
    ?.split(/\r?\n/)
    .map(line => line.trim())
    .filter(line => line.length > 0);

  if (!lines || lines.length === 0) return null;

  return lines.map(line => `${line}\r`).join('');
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => {
    window.setTimeout(resolve, ms);
  });
}

async function runStartupCommand(
  session: Session,
  generation: number,
  options: SessionStartOptions,
): Promise<void> {
  const input = startupCommandInput(options.command);
  if (!input) return;

  await delay(STARTUP_COMMAND_DELAY_MS);

  if (
    sessions.get(options.paneId) !== session ||
    session.startGeneration !== generation ||
    session.status !== 'running'
  ) {
    return;
  }

  await writeTerminal(options.paneId, input);
}

export function getCommandLog(paneId: string): string[] {
  return [...(sessions.get(paneId)?.commandLog ?? [])];
}

if (import.meta.env.DEV) {
  // Debug hook for driving/inspecting sessions from the devtools console.
  (window as unknown as Record<string, unknown>).__devpaneSessions = {
    dump: () =>
      [...sessions.entries()].map(([paneId, session]) => ({
        paneId,
        status: session.status,
        commandLog: [...session.commandLog],
      })),
  };
}

function setStatus(session: Session, status: SessionStatus) {
  if (session.status === status) return;
  session.status = status;
  session.statusSubscribers.forEach(subscriber => subscriber(status));
}

export function getStatus(paneId: string): SessionStatus {
  return sessions.get(paneId)?.status ?? 'idle';
}

export function getBuffer(paneId: string): string {
  return sessions.get(paneId)?.buffer ?? '';
}

export function subscribe(
  paneId: string,
  onData: (data: string) => void,
  onStatus: (status: SessionStatus) => void,
): () => void {
  const session = getOrCreate(paneId);
  session.dataSubscribers.add(onData);
  session.statusSubscribers.add(onStatus);

  return () => {
    session.dataSubscribers.delete(onData);
    session.statusSubscribers.delete(onStatus);
  };
}

export async function start(options: SessionStartOptions): Promise<void> {
  const session = getOrCreate(options.paneId);
  if (session.status === 'running') return;
  const generation = session.startGeneration + 1;

  session.lastStart = options;
  session.buffer = '';
  session.commandLog = [];
  session.startGeneration = generation;

  if (!session.unlistenOutput) {
    session.unlistenOutput = await listenTerminalOutput(options.paneId, output => {
      session.buffer = (session.buffer + output.data).slice(-bufferLimit);
      session.dataSubscribers.forEach(subscriber => subscriber(output.data));
    });
  }

  if (!session.unlistenExit) {
    session.unlistenExit = await listenTerminalExit(options.paneId, () => {
      setStatus(session, 'exited');
    });
  }

  await startTerminal({
    paneId: options.paneId,
    paneName: options.paneName,
    cwd: options.cwd,
    shell: options.shell,
    rows: options.rows,
    cols: options.cols,
  });
  setStatus(session, 'running');
  await runStartupCommand(session, generation, options);
}

export async function restart(paneId: string, rows: number, cols: number): Promise<void> {
  const session = sessions.get(paneId);
  if (!session?.lastStart || session.status === 'running') return;

  await start({ ...session.lastStart, rows, cols });
}

export function write(paneId: string, data: string): Promise<void> {
  const session = sessions.get(paneId);
  if (!session || session.status !== 'running') return Promise.resolve();
  return writeTerminal(paneId, data);
}

export function resize(paneId: string, rows: number, cols: number): Promise<void> {
  if (getStatus(paneId) !== 'running') return Promise.resolve();
  return resizeTerminal(paneId, rows, cols);
}

export async function stop(paneId: string): Promise<void> {
  const session = sessions.get(paneId);
  if (!session) return;

  session.unlistenOutput?.();
  session.unlistenExit?.();
  sessions.delete(paneId);

  if (session.status === 'running') {
    await stopTerminal(paneId).catch(error => {
      console.error(`stopTerminal(${paneId}) failed`, error);
    });
  }
}

export async function stopAll(): Promise<void> {
  await Promise.all([...sessions.keys()].map(paneId => stop(paneId)));
}
