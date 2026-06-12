<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';
import * as sessions from '../terminalSessions';
import { onWindowResized } from '../api/window';
import type { WorkspacePaneState } from '../types';

const props = defineProps<{
  pane: WorkspacePaneState;
  root: string | null;
  scrollback: number;
}>();

const emit = defineEmits<{
  focused: [paneId: string];
  close: [paneId: string];
}>();

const host = ref<HTMLElement | null>(null);
const status = ref<sessions.SessionStatus>(sessions.getStatus(props.pane.id));
let terminal: Terminal | null = null;
let fit: FitAddon | null = null;
let resizeObserver: ResizeObserver | null = null;
let unsubscribe: (() => void) | null = null;
let unlistenWindow: (() => void) | null = null;

function fitAndResize() {
  if (!terminal || !fit) return;
  fit.fit();
  void sessions.resize(props.pane.id, terminal.rows, terminal.cols).catch(error => {
    console.error('resizeTerminal failed', error);
  });
}

function scheduleFit() {
  window.requestAnimationFrame(() => {
    window.setTimeout(fitAndResize, 25);
  });
}

function startOptions(): sessions.SessionStartOptions {
  return {
    paneId: props.pane.id,
    paneName: props.pane.name,
    cwd: props.pane.cwd ?? props.root,
    shell: props.pane.shell,
    command: props.pane.command || null,
    rows: terminal?.rows ?? 24,
    cols: terminal?.cols ?? 80,
  };
}

function reportError(error: unknown) {
  console.error('terminal error', error);
  terminal?.writeln(`\r\n[terminal error: ${String(error)}]\r\n`);
}

function startSession() {
  fitAndResize();
  sessions.start(startOptions()).catch(reportError);
}

function restartSession() {
  terminal?.reset();
  fitAndResize();
  sessions
    .restart(props.pane.id, terminal?.rows ?? 24, terminal?.cols ?? 80)
    .catch(reportError);
}

async function mountTerminal() {
  if (!host.value || terminal) return;

  terminal = new Terminal({
    cursorBlink: true,
    fontFamily: 'Consolas, "Cascadia Mono", monospace',
    fontSize: 13,
    scrollback: props.scrollback,
    theme: {
      background: '#05080d',
      foreground: '#e6edf3',
      cursor: '#58a6ff',
    },
  });
  fit = new FitAddon();
  terminal.loadAddon(fit);
  terminal.open(host.value);
  // xterm 6 removed Terminal.onFocus; track focus via the DOM instead.
  host.value.addEventListener('focusin', () => {
    emit('focused', props.pane.id);
  });
  terminal.onData(data => {
    void sessions.write(props.pane.id, data).catch(error => {
      console.error('writeTerminal failed', error);
    });
  });

  const replay = sessions.getBuffer(props.pane.id);
  if (replay) {
    terminal.write(replay);
  }

  unsubscribe = sessions.subscribe(
    props.pane.id,
    data => terminal?.write(data),
    next => {
      status.value = next;
    },
  );
  unlistenWindow = await onWindowResized(scheduleFit);

  resizeObserver = new ResizeObserver(scheduleFit);
  resizeObserver.observe(host.value);
  scheduleFit();

  if (sessions.getStatus(props.pane.id) === 'idle' && props.pane.autoStart) {
    startSession();
  }
}

onMounted(() => {
  mountTerminal().catch(reportError);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  unsubscribe?.();
  unlistenWindow?.();
  // The PTY session intentionally survives unmount; layout changes must not
  // kill running terminals. Sessions stop via terminalSessions.stop().
  terminal?.dispose();
  terminal = null;
});
</script>

<template>
  <section class="terminal-pane">
    <div ref="host" class="terminal-host"></div>

    <div v-if="status === 'exited'" class="terminal-overlay">
      <span class="terminal-overlay-text">[process exited]</span>
      <div class="terminal-overlay-actions">
        <button type="button" class="primary-btn" @click="restartSession">Restart</button>
        <button type="button" class="secondary-btn" @click="emit('close', pane.id)">Close</button>
      </div>
    </div>

    <div v-else-if="status === 'idle' && !pane.autoStart" class="terminal-overlay">
      <span class="terminal-overlay-text">{{ pane.name }} (auto start off)</span>
      <button type="button" class="primary-btn" @click="startSession">Start</button>
    </div>
  </section>
</template>
