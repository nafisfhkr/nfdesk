import { invoke } from '@tauri-apps/api/core';

const VAULT_PATH_KEY = 'nfdesk-vault-path';
const DAILY_NOTES_FOLDER_KEY = 'nfdesk-daily-notes-folder';
const TASKS_FOLDER_KEY = 'nfdesk-tasks-folder';

const DEFAULT_DAILY_NOTES_FOLDER = 'Daily Notes';
const DEFAULT_TASKS_FOLDER = 'Tasks';

export interface MarkdownTask {
  id: string;
  title: string;
  completed: boolean;
}

export function getVaultPath(): string {
  return localStorage.getItem(VAULT_PATH_KEY) ?? '';
}

export function setVaultPath(path: string) {
  localStorage.setItem(VAULT_PATH_KEY, path);
}

export function getDailyNotesFolder(): string {
  return localStorage.getItem(DAILY_NOTES_FOLDER_KEY) || DEFAULT_DAILY_NOTES_FOLDER;
}

export function setDailyNotesFolder(folder: string) {
  localStorage.setItem(DAILY_NOTES_FOLDER_KEY, folder.trim() || DEFAULT_DAILY_NOTES_FOLDER);
}

export function getTasksFolder(): string {
  return localStorage.getItem(TASKS_FOLDER_KEY) || DEFAULT_TASKS_FOLDER;
}

export function setTasksFolder(folder: string) {
  localStorage.setItem(TASKS_FOLDER_KEY, folder.trim() || DEFAULT_TASKS_FOLDER);
}

export function todayFilename(): string {
  const d = new Date();
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const dd = String(d.getDate()).padStart(2, '0');
  return `${d.getFullYear()}-${mm}-${dd}.md`;
}

export function timeStamp(d = new Date()): string {
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

function requireVaultPath(): string {
  const vaultPath = getVaultPath();
  if (!vaultPath) {
    throw new Error('Set your Obsidian vault path in Settings first.');
  }
  return vaultPath;
}

export async function readTasksFromVault(filename?: string): Promise<MarkdownTask[]> {
  return invoke<MarkdownTask[]>('read_markdown_tasks', {
    vaultPath: requireVaultPath(),
    folder: getTasksFolder(),
    filename: filename ?? todayFilename(),
  });
}

export async function saveTasksToVault(tasks: MarkdownTask[], filename?: string): Promise<boolean> {
  return invoke<boolean>('save_markdown_tasks', {
    vaultPath: requireVaultPath(),
    folder: getTasksFolder(),
    filename: filename ?? todayFilename(),
    tasks,
  });
}

export async function appendDailyNote(content: string, filename?: string): Promise<boolean> {
  return invoke<boolean>('append_to_markdown', {
    vaultPath: requireVaultPath(),
    folder: getDailyNotesFolder(),
    filename: filename ?? todayFilename(),
    content,
  });
}

// Kept for callers that only need a raw append; appends to the vault root.
export async function appendToMarkdown(content: string, filename?: string) {
  return invoke<boolean>('append_to_markdown', {
    vaultPath: requireVaultPath(),
    folder: '',
    filename: filename ?? todayFilename(),
    content,
  });
}
