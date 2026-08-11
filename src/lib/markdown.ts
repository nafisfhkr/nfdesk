import { invoke } from '@tauri-apps/api/core';

const VAULT_PATH_KEY = 'nfdesk-vault-path';

export function getVaultPath(): string {
  return localStorage.getItem(VAULT_PATH_KEY) ?? '';
}

export function setVaultPath(path: string) {
  localStorage.setItem(VAULT_PATH_KEY, path);
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

export async function appendToMarkdown(content: string, filename?: string) {
  const vaultPath = getVaultPath();
  if (!vaultPath) {
    throw new Error('Set your Obsidian vault path in Settings first.');
  }
  return invoke<boolean>('append_to_markdown', {
    vaultPath,
    filename: filename ?? todayFilename(),
    content,
  });
}
