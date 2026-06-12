import type {
  SplitDirection,
  WorkspaceLayoutNode,
  WorkspaceLayoutPane,
  WorkspaceLayoutSplit,
  WorkspacePaneState,
} from './types';

export type LayoutPath = number[];

export function createPaneNode(pane: string, size: number | null = null): WorkspaceLayoutPane {
  return {
    kind: 'pane',
    pane,
    size,
  };
}

export function collectPaneIds(node: WorkspaceLayoutNode): string[] {
  if (node.kind === 'pane') {
    return [node.pane];
  }

  return node.children.flatMap(child => collectPaneIds(child));
}

export function firstPaneId(node: WorkspaceLayoutNode): string | null {
  if (node.kind === 'pane') {
    return node.pane;
  }

  for (const child of node.children) {
    const paneId = firstPaneId(child);
    if (paneId) return paneId;
  }

  return null;
}

export function findPanePath(node: WorkspaceLayoutNode, paneId: string, path: LayoutPath = []): LayoutPath | null {
  if (node.kind === 'pane') {
    return node.pane === paneId ? path : null;
  }

  for (let index = 0; index < node.children.length; index += 1) {
    const childPath = findPanePath(node.children[index], paneId, [...path, index]);
    if (childPath) return childPath;
  }

  return null;
}

export function insertPaneIntoLayout(
  node: WorkspaceLayoutNode,
  activePaneId: string,
  direction: SplitDirection,
  newPaneId: string,
): WorkspaceLayoutNode {
  const path = findPanePath(node, activePaneId);
  if (!path) {
    return node;
  }

  return insertPaneAtPath(node, path, direction, newPaneId);
}

function insertPaneAtPath(
  node: WorkspaceLayoutNode,
  path: LayoutPath,
  direction: SplitDirection,
  newPaneId: string,
): WorkspaceLayoutNode {
  if (path.length === 0) {
    if (node.kind !== 'pane') {
      return node;
    }

    const size = node.size ?? 100;
    const childSize = Math.max(1, Math.floor(size / 2));
    const otherSize = Math.max(1, size - childSize);
    return {
      kind: 'split',
      direction,
      size,
      children: [
        {
          ...node,
          size: childSize,
        },
        createPaneNode(newPaneId, otherSize),
      ],
    };
  }

  if (node.kind !== 'split') {
    return node;
  }

  const [index, ...rest] = path;
  const child = node.children[index];
  if (!child) {
    return node;
  }

  if (rest.length === 0 && child.kind === 'pane' && node.direction === direction) {
    const size = child.size ?? 100;
    const childSize = Math.max(1, Math.floor(size / 2));
    const otherSize = Math.max(1, size - childSize);
    const children = [...node.children];
    children[index] = {
      ...child,
      size: childSize,
    };
    children.splice(index + 1, 0, createPaneNode(newPaneId, otherSize));
    return {
      ...node,
      children,
    };
  }

  const nextChild = insertPaneAtPath(child, rest, direction, newPaneId);
  if (nextChild === child) {
    return node;
  }

  const children = [...node.children];
  children[index] = nextChild;
  return {
    ...node,
    children,
  };
}

export function removePaneFromLayout(
  node: WorkspaceLayoutNode,
  paneId: string,
): WorkspaceLayoutNode | null {
  if (node.kind === 'pane') {
    return node.pane === paneId ? null : node;
  }

  const children = node.children
    .map(child => removePaneFromLayout(child, paneId))
    .filter((child): child is WorkspaceLayoutNode => child !== null);

  if (children.length === 0) {
    return null;
  }

  if (children.length === 1) {
    return children[0];
  }

  return {
    ...node,
    children,
  };
}

export function updateSplitSizes(
  node: WorkspaceLayoutNode,
  path: LayoutPath,
  sizes: number[],
): WorkspaceLayoutNode {
  if (path.length === 0) {
    if (node.kind !== 'split') {
      return node;
    }

    return {
      ...node,
      children: node.children.map((child, index) => ({
        ...child,
        size: sizes[index] ?? child.size,
      })),
    };
  }

  if (node.kind !== 'split') {
    return node;
  }

  const [index, ...rest] = path;
  const child = node.children[index];
  if (!child) {
    return node;
  }

  const nextChild = updateSplitSizes(child, rest, sizes);
  if (nextChild === child) {
    return node;
  }

  const children = [...node.children];
  children[index] = nextChild;
  return {
    ...node,
    children,
  };
}

export function splitChildSizes(node: WorkspaceLayoutSplit): number[] {
  return node.children.map(child => child.size ?? 1);
}

export function normalizePaneSizes(node: WorkspaceLayoutNode): WorkspaceLayoutNode {
  if (node.kind === 'pane') {
    return node;
  }

  const children = node.children.map(child => normalizePaneSizes(child));
  if (children.length === 1) {
    return children[0];
  }

  return {
    ...node,
    children,
  };
}

export function paneMapById(panes: WorkspacePaneState[]): Record<string, WorkspacePaneState> {
  return Object.fromEntries(panes.map(pane => [pane.id, pane]));
}
