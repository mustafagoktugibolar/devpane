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
}>();

const emit = defineEmits<{
  'new-session': [];
  sessions: [];
  save: [];
  'add-terminal': [];
  'add-horizontal': [];
  'add-vertical': [];
  'close-terminal': [];
}>();

const openMenu = ref<string | null>(null);

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
</script>

<template>
  <header class="titlebar" @click="closeMenus">
    <nav class="titlebar-menu" @click.stop>
      <div class="titlebar-menu-item">
        <button class="titlebar-menu-btn" type="button" @click="toggleMenu('file')">File</button>
        <div class="titlebar-menu-panel" :class="{ open: openMenu === 'file' }">
          <button type="button" @click="run(() => emit('new-session'))">New Session</button>
          <button type="button" @click="run(() => emit('sessions'))">Sessions</button>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('save'))">Save</button>
        </div>
      </div>

      <div class="titlebar-menu-item">
        <button class="titlebar-menu-btn" type="button" @click="toggleMenu('terminal')">Terminal</button>
        <div class="titlebar-menu-panel" :class="{ open: openMenu === 'terminal' }">
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('add-terminal'))">Add Terminal</button>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('add-horizontal'))">Add Horizontal</button>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('add-vertical'))">Add Vertical</button>
          <button type="button" :disabled="!hasWorkspace" @click="run(() => emit('close-terminal'))">Close Terminal</button>
        </div>
      </div>
    </nav>

    <div class="titlebar-drag" data-tauri-drag-region @pointerdown="onDrag"></div>

    <div class="titlebar-controls" @click.stop>
      <button class="titlebar-btn" type="button" @click="runWindowAction(minimizeWindow)">&#x2013;</button>
      <button class="titlebar-btn" type="button" @click="runWindowAction(toggleMaximizeWindow)">&#x25A1;</button>
      <button class="titlebar-btn close" type="button" @click="runWindowAction(closeWindow)">&#x2715;</button>
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
