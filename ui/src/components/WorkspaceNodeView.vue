<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import TerminalPane from './TerminalPane.vue';
import { firstPaneId, splitChildSizes, type LayoutPath } from '../layout';
import type { WorkspaceLayoutNode, WorkspacePaneState } from '../types';

defineOptions({ name: 'WorkspaceNodeView' });

const props = defineProps<{
  node: WorkspaceLayoutNode;
  panesById: Record<string, WorkspacePaneState>;
  root: string | null;
  activePaneId: string | null;
  scrollback: number;
  path: LayoutPath;
}>();

const emit = defineEmits<{
  'focus-pane': [paneId: string];
  'close-pane': [paneId: string];
  'resize-split': [payload: { path: LayoutPath; sizes: number[] }];
}>();

// Key children by what they contain, not their position, so removing a
// sibling never makes Vue reuse a pane component for a different pane.
function childKey(child: WorkspaceLayoutNode): string {
  return child.kind === 'pane' ? `pane-${child.pane}` : `split-${firstPaneId(child) ?? 'empty'}`;
}

const splitHost = ref<HTMLElement | null>(null);
let dragCleanup: (() => void) | null = null;

const splitSizes = computed(() => (props.node.kind === 'split' ? splitChildSizes(props.node) : []));

function childStyle(index: number) {
  const node = props.node;
  if (node.kind !== 'split') {
    return {};
  }

  const size = splitSizes.value[index] ?? 1;
  return {
    flex: `${size} 1 0%`,
    minWidth: node.direction === 'vertical' ? '0' : undefined,
    minHeight: node.direction === 'horizontal' ? '0' : undefined,
  };
}

function startResize(index: number, event: PointerEvent) {
  const node = props.node;
  if (node.kind !== 'split' || !splitHost.value) return;

  const gutter = event.currentTarget as HTMLElement | null;
  if (!gutter) return;

  event.preventDefault();
  gutter.setPointerCapture(event.pointerId);

  const startSizes = splitSizes.value.slice();
  const rect = splitHost.value.getBoundingClientRect();
  const axisSize = node.direction === 'vertical' ? rect.width : rect.height;
  const startPosition = node.direction === 'vertical' ? event.clientX : event.clientY;
  const totalWeight = startSizes.reduce((sum, size) => sum + size, 0) || 1;
  const minWeight = Math.max(Math.round(totalWeight * 0.08), 10);

  const applyResize = (clientX: number, clientY: number) => {
    const currentPosition = node.direction === 'vertical' ? clientX : clientY;
    const delta = currentPosition - startPosition;
    const deltaWeight = axisSize > 0 ? (delta / axisSize) * totalWeight : 0;

    const next = startSizes.slice();
    let left = startSizes[index] + deltaWeight;
    left = Math.max(minWeight, Math.min(totalWeight - minWeight, left));
    let leftRounded = Math.round(left);
    let rightRounded = Math.round(totalWeight) - leftRounded;

    if (rightRounded < minWeight) {
      rightRounded = minWeight;
      leftRounded = Math.round(totalWeight) - rightRounded;
    }

    next[index] = leftRounded;
    next[index + 1] = rightRounded;
    emit('resize-split', { path: props.path, sizes: next });
  };

  const onMove = (moveEvent: PointerEvent) => {
    applyResize(moveEvent.clientX, moveEvent.clientY);
  };

  const onUp = () => {
    window.removeEventListener('pointermove', onMove);
    window.removeEventListener('pointerup', onUp);
    dragCleanup = null;
  };

  dragCleanup = onUp;
  window.addEventListener('pointermove', onMove);
  window.addEventListener('pointerup', onUp, { once: true });
}

onBeforeUnmount(() => {
  dragCleanup?.();
});
</script>

<template>
  <template v-if="node.kind === 'pane'">
    <TerminalPane
      v-if="panesById[node.pane]"
      :pane="panesById[node.pane]"
      :root="root"
      :scrollback="scrollback"
      :active="activePaneId === node.pane"
      @focused="emit('focus-pane', $event)"
      @close="emit('close-pane', $event)"
    />
    <section v-else class="terminal-pane terminal-pane-missing">
      <div class="terminal-host"></div>
    </section>
  </template>

  <section
    v-else
    ref="splitHost"
    class="workspace-split"
    :class="node.direction"
  >
    <template v-for="(child, index) in node.children" :key="childKey(child)">
      <div class="workspace-split-child" :style="childStyle(index)">
        <WorkspaceNodeView
          :node="child"
          :panes-by-id="panesById"
          :root="root"
          :active-pane-id="activePaneId"
          :scrollback="scrollback"
          :path="[...path, index]"
          @focus-pane="emit('focus-pane', $event)"
          @close-pane="emit('close-pane', $event)"
          @resize-split="emit('resize-split', $event)"
        />
      </div>
      <div
        v-if="index < node.children.length - 1"
        class="workspace-split-gutter"
        :class="node.direction"
        @pointerdown="startResize(index, $event)"
      ></div>
    </template>
  </section>
</template>
