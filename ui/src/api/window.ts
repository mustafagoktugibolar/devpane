import { getCurrentWindow } from '@tauri-apps/api/window';
import type { ResizeDirection } from '../types';

const appWindow = getCurrentWindow();

export function setWindowTitle(name?: string): Promise<void> {
  return appWindow.setTitle(name ? `DevPane - ${name}` : 'DevPane');
}

export function minimizeWindow(): Promise<void> {
  return appWindow.minimize();
}

export function toggleMaximizeWindow(): Promise<void> {
  return appWindow.toggleMaximize();
}

export function closeWindow(): Promise<void> {
  return appWindow.close();
}

export function startWindowDrag(): Promise<void> {
  return appWindow.startDragging();
}

export function startWindowResize(direction: ResizeDirection): Promise<void> {
  return appWindow.startResizeDragging(direction);
}

export function onWindowResized(handler: () => void): Promise<() => void> {
  return appWindow.onResized(handler);
}
