import './style.css';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface RecentSession {
  path: string;
  name: string;
  last_opened: number;
}

const appWindow = getCurrentWindow();
let sessions: RecentSession[] = [];
let selectedIndex = 0;
let keyHandler: ((e: KeyboardEvent) => void) | null = null;

function unbindKeys() {
  if (keyHandler) {
    document.removeEventListener('keydown', keyHandler);
    keyHandler = null;
  }
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// ── Titlebar ──────────────────────────────────────────────────────────────────

function initTitlebar() {
  const titlebar = document.createElement('div');
  titlebar.className = 'titlebar';
  titlebar.setAttribute('data-tauri-drag-region', '');
  titlebar.innerHTML = `
    <span class="titlebar-title">DevPane</span>
    <div class="titlebar-controls">
      <button class="titlebar-btn" id="btn-min">&#x2013;</button>
      <button class="titlebar-btn" id="btn-max">&#x25A1;</button>
      <button class="titlebar-btn close" id="btn-close">&#x2715;</button>
    </div>`;
  document.querySelector('#app')!.prepend(titlebar);

  document.getElementById('btn-min')?.addEventListener('click', () => appWindow.minimize());
  document.getElementById('btn-max')?.addEventListener('click', () => appWindow.toggleMaximize());
  document.getElementById('btn-close')?.addEventListener('click', () => appWindow.close());
}

function setTitlebarSession(name?: string) {
  const el = document.querySelector('.titlebar-title');
  if (!el) return;
  el.textContent = name ? `DevPane  ·  ${name}` : 'DevPane';
}

// ── Session Picker ────────────────────────────────────────────────────────────

async function showSessionPicker() {
  setTitlebarSession();
  sessions = await invoke<RecentSession[]>('list_recent_sessions');
  selectedIndex = 0;
  renderPicker();
}

function renderPicker() {
  const total = 1 + sessions.length;

  const newItem = `
    <div class="picker-item ${selectedIndex === 0 ? 'selected' : ''}" data-index="0">
      <span class="picker-new-icon">+</span>
      <div class="picker-item-info">
        <span class="picker-item-name">New Session</span>
      </div>
    </div>`;

  const recentItems = sessions.map((s, i) => `
    <div class="picker-item ${selectedIndex === i + 1 ? 'selected' : ''}" data-index="${i + 1}">
      <div class="picker-item-info">
        <span class="picker-item-name">${escapeHtml(s.name)}</span>
        <span class="picker-item-path">${escapeHtml(s.path)}</span>
      </div>
    </div>`).join('');

  document.querySelector('#content')!.innerHTML = `
    <div class="picker">
      <div class="picker-list">
        ${newItem}
        ${sessions.length > 0 ? `<div class="picker-divider"></div>${recentItems}` : ''}
      </div>
      <div class="picker-hint">&#x2191; &#x2193; &nbsp;&middot;&nbsp; &#x23CE; open</div>
    </div>`;

  document.querySelector('.picker-item.selected')?.scrollIntoView({ block: 'nearest' });

  document.querySelectorAll<HTMLElement>('.picker-item').forEach(el => {
    el.addEventListener('click', () => {
      selectedIndex = parseInt(el.dataset.index ?? '0');
      openSelected();
    });
  });

  unbindKeys();
  keyHandler = (e: KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % total;
      renderPicker();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + total) % total;
      renderPicker();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      openSelected();
    }
  };
  document.addEventListener('keydown', keyHandler);
}

async function openSelected() {
  unbindKeys();
  if (selectedIndex === 0) {
    showWorkspace(null);
  } else {
    const session = sessions[selectedIndex - 1];
    await invoke('add_recent_session', { path: session.path, name: session.name });
    showWorkspace(session);
  }
}

// ── Workspace ────────────────────────────────────────────────────────────────

function showWorkspace(session: RecentSession | null) {
  setTitlebarSession(session?.name);

  document.querySelector('#content')!.innerHTML = `
    <div class="workspace">
      <div class="workspace-body">
        <span class="workspace-empty">No terminals yet</span>
      </div>
    </div>`;
}

// ── Init ──────────────────────────────────────────────────────────────────────

document.querySelector('#app')!.innerHTML = `
  <div id="content" style="flex:1;display:flex;flex-direction:column;overflow:hidden;"></div>`;

initTitlebar();
showSessionPicker();
