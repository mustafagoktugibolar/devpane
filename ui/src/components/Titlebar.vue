<script setup lang="ts">
import { ref } from 'vue';
import {
  closeWindow,
  minimizeWindow,
  startWindowDrag,
  startWindowResize,
  toggleMaximizeWindow,
} from '../api/window';
import type { ResizeDirection } from '../types';

defineProps<{
  hasWorkspace: boolean;
  dirty?: boolean;
}>();

const emit = defineEmits<{
  'new-session': [];
  sessions: [];
  save: [];
  'split-right': [shell: string | null];
  'split-down': [shell: string | null];
  'close-terminal': [];
}>();

const openMenu = ref<string | null>(null);
const selectedShell = ref<string | null>(null); // null = PowerShell default

const SHELLS: { label: string; value: string | null }[] = [
  { label: 'PowerShell', value: null },
  { label: 'cmd', value: 'cmd' },
];

function toggleMenu(menu: string) {
  openMenu.value = openMenu.value === menu ? null : menu;
}

function closeMenus() {
  openMenu.value = null;
}

function run(action: () => void) {
  closeMenus();
  action();
}

function runWindowAction(action: () => Promise<void>) {
  void action().catch(() => undefined);
}

function onDrag(event: PointerEvent) {
  if (event.button !== 0) return;
  void startWindowDrag().catch(() => undefined);
}

function onResize(event: PointerEvent, direction: ResizeDirection) {
  if (event.button !== 0) return;
  event.preventDefault();
  void startWindowResize(direction).catch(() => undefined);
}

function shellDisplayName(value: string | null): string {
  return SHELLS.find(s => s.value === value)?.label ?? 'PowerShell';
}
</script>

<template>
  <header class="titlebar" @click="closeMenus">
    <nav class="titlebar-menu" @click.stop>
      <!-- File menu -->
      <div class="titlebar-menu-item">
        <button class="titlebar-menu-btn" type="button" @click="toggleMenu('file')">File</button>
        <div class="titlebar-menu-panel" :class="{ open: openMenu === 'file' }">
          <button type="button" @click="run(() => emit('new-session'))">New Session</button>
          <button type="button" @click="run(() => emit('sessions'))">Sessions</button>
        </div>
      </div>

      <!-- Terminal menu -->
      <div class="titlebar-menu-item">
        <button class="titlebar-menu-btn" type="button" @click="toggleMenu('terminal')">Terminal</button>
        <div class="titlebar-menu-panel" :class="{ open: openMenu === 'terminal' }">
          <div class="menu-section-label">Split Right</div>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('split-right', null))">
            <span class="menu-shell-icon">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <rect x="2" y="2" width="12" height="12" rx="2"/>
                <line x1="9" y1="2" x2="9" y2="14"/>
              </svg>
            </span>
            Split Right — PowerShell
          </button>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('split-right', 'cmd'))">
            <span class="menu-shell-icon">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <rect x="2" y="2" width="12" height="12" rx="2"/>
                <line x1="9" y1="2" x2="9" y2="14"/>
              </svg>
            </span>
            Split Right — cmd
          </button>
          <div class="menu-divider"></div>
          <div class="menu-section-label">Split Down</div>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('split-down', null))">
            <span class="menu-shell-icon">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <rect x="2" y="2" width="12" height="12" rx="2"/>
                <line x1="2" y1="9" x2="14" y2="9"/>
              </svg>
            </span>
            Split Down — PowerShell
          </button>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('split-down', 'cmd'))">
            <span class="menu-shell-icon">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
                <rect x="2" y="2" width="12" height="12" rx="2"/>
                <line x1="2" y1="9" x2="14" y2="9"/>
              </svg>
            </span>
            Split Down — cmd
          </button>
          <div class="menu-divider"></div>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('close-terminal'))">Close Terminal</button>
        </div>
      </div>
    </nav>

    <!-- Quick-action buttons -->
    <div class="titlebar-actions" @click.stop>
      <!-- Shell picker dropdown -->
      <div class="titlebar-menu-item">
        <button
          class="titlebar-action-btn shell-picker-btn"
          type="button"
          :disabled="!hasWorkspace"
          :title="`Default shell: ${shellDisplayName(selectedShell)}`"
          @click="toggleMenu('shell-picker')"
        >
          <span class="shell-picker-label">{{ shellDisplayName(selectedShell) }}</span>
          <svg width="8" height="8" viewBox="0 0 10 6" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="1,1 5,5 9,1"/>
          </svg>
        </button>
        <div class="titlebar-menu-panel shell-picker-panel" :class="{ open: openMenu === 'shell-picker' }">
          <button
            v-for="shell in SHELLS"
            :key="String(shell.value)"
            type="button"
            :class="{ active: selectedShell === shell.value }"
            @click="run(() => { selectedShell = shell.value; })"
          >
            {{ shell.label }}
          </button>
        </div>
      </div>

      <!-- Split Right quick button -->
      <button
        class="titlebar-action-btn"
        type="button"
        :disabled="!hasWorkspace"
        :title="`Split Right (${shellDisplayName(selectedShell)})`"
        @click="emit('split-right', selectedShell)"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <rect x="2" y="2" width="12" height="12" rx="2"/>
          <line x1="9" y1="2" x2="9" y2="14"/>
        </svg>
      </button>

      <!-- Split Down quick button -->
      <button
        class="titlebar-action-btn"
        type="button"
        :disabled="!hasWorkspace"
        :title="`Split Down (${shellDisplayName(selectedShell)})`"
        @click="emit('split-down', selectedShell)"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <rect x="2" y="2" width="12" height="12" rx="2"/>
          <line x1="2" y1="9" x2="14" y2="9"/>
        </svg>
      </button>

      <!-- Save quick button -->
      <button
        class="titlebar-action-btn"
        type="button"
        :disabled="!hasWorkspace"
        :class="{ 'save-dirty': dirty }"
        title="Save workspace"
        @click="emit('save')"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M13 13H3a1 1 0 0 1-1-1V4l3-3h7a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1z"/>
          <rect x="5" y="9" width="6" height="4" rx="0.5"/>
          <rect x="5.5" y="2" width="4" height="3" rx="0.5"/>
        </svg>
        <span v-if="dirty" class="save-dot" aria-hidden="true"></span>
      </button>
    </div>

    <div class="titlebar-drag" data-tauri-drag-region @pointerdown="onDrag"></div>

    <div class="titlebar-controls" @click.stop>
      <!-- Minimize -->
      <button class="titlebar-btn" type="button" title="Minimize" @click="runWindowAction(minimizeWindow)">
        <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
          <rect width="10" height="1"/>
        </svg>
      </button>
      <!-- Maximize / Restore -->
      <button class="titlebar-btn" type="button" title="Maximize" @click="runWindowAction(toggleMaximizeWindow)">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.1">
          <rect x="0.55" y="0.55" width="8.9" height="8.9"/>
        </svg>
      </button>
      <!-- Close -->
      <button class="titlebar-btn close" type="button" title="Close" @click="runWindowAction(closeWindow)">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
          <line x1="0.5" y1="0.5" x2="9.5" y2="9.5"/>
          <line x1="9.5" y1="0.5" x2="0.5" y2="9.5"/>
        </svg>
      </button>
    </div>

    <div class="resize-hitbox resize-n" @pointerdown="event => onResize(event, 'North')"></div>
    <div class="resize-hitbox resize-e" @pointerdown="event => onResize(event, 'East')"></div>
    <div class="resize-hitbox resize-s" @pointerdown="event => onResize(event, 'South')"></div>
    <div class="resize-hitbox resize-w" @pointerdown="event => onResize(event, 'West')"></div>
    <div class="resize-hitbox resize-ne" @pointerdown="event => onResize(event, 'NorthEast')"></div>
    <div class="resize-hitbox resize-nw" @pointerdown="event => onResize(event, 'NorthWest')"></div>
    <div class="resize-hitbox resize-se" @pointerdown="event => onResize(event, 'SouthEast')"></div>
    <div class="resize-hitbox resize-sw" @pointerdown="event => onResize(event, 'SouthWest')"></div>
  </header>
</template>
