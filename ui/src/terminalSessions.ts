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
  lineBuffer: string;
  lineTainted: boolean;
  escapeBuffer: string | null;
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
      lineBuffer: '',
      lineTainted: false,
      escapeBuffer: null,
      startGeneration: 0,
    };
    sessions.set(paneId, session);
  }
  return session;
}

// Escape sequences that carry no line edits: focus in/out reports and the
// bracketed paste markers (the pasted text between them arrives as plain
// characters and is captured normally).
const HARMLESS_ESCAPES = new Set(['[I', '[O', '[200~', '[201~']);

function finishEscape(session: Session, sequence: string) {
  session.escapeBuffer = null;
  if (!HARMLESS_ESCAPES.has(sequence)) {
    // Arrow keys, history recall, delete, etc. edit the line in ways we
    // cannot observe — stop trusting it.
    session.lineTainted = true;
  }
}

// Reconstructs typed command lines from the raw input stream so a workspace
// save can offer "what this terminal did" as startup commands. Lines touched
// by tab completion, history recall, or cursor-movement escape sequences
// cannot be observed reliably and are skipped.
function captureInput(session: Session, data: string) {
  for (const ch of data) {
    if (session.escapeBuffer !== null) {
      session.escapeBuffer += ch;
      const sequence = session.escapeBuffer;

      if (sequence.length === 1) {
        // CSI (ESC [) and SS3 (ESC O) sequences continue; any other
        // two-character escape ends here.
        if (ch !== '[' && ch !== 'O') {
          finishEscape(session, sequence);
        }
      } else if (sequence[0] === '[') {
        // CSI ends at a final byte in @ ... ~.
        if (ch >= '@' && ch <= '~') {
          finishEscape(session, sequence);
        }
      } else {
        finishEscape(session, sequence);
      }
      continue;
    }

    if (ch === '\x1b') {
      session.escapeBuffer = '';
    } else if (ch === '\r' || ch === '\n') {
      const line = session.lineBuffer.trim();
      if (line && !session.lineTainted) {
        session.commandLog.push(line);
      }
      session.lineBuffer = '';
      session.lineTainted = false;
    } else if (ch === '\x7f' || ch === '\b') {
      session.lineBuffer = session.lineBuffer.slice(0, -1);
    } else if (ch === '\x03' || ch === '\x15') {
      // Ctrl+C / Ctrl+U discard the current line.
      session.lineBuffer = '';
      session.lineTainted = false;
    } else if (ch < ' ') {
      session.lineTainted = true;
    } else {
      session.lineBuffer += ch;
    }
  }
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
        lineBuffer: session.lineBuffer,
        lineTainted: session.lineTainted,
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
  session.lineBuffer = '';
  session.lineTainted = false;
  session.escapeBuffer = null;
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
  captureInput(session, data);
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
