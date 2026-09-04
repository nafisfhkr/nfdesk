import { invoke } from '@tauri-apps/api/core';

export interface MarkdownTask {
  id: string;
  title: string;
  completed: boolean;
}

export interface AppSettingsResponse {
  vault_configured: boolean;
  vault_path: string | null;
}

export interface VaultWarning {
  code: string;
  message: string;
}

export interface VaultPreview {
  canonical_vault_path: string;
  is_obsidian_vault: boolean;
  directories_to_create: string[];
  existing_directories: string[];
  warnings: VaultWarning[];
}

export interface VaultSetupResult {
  vault_path: string;
  manifest_created: boolean;
  created_directories: string[];
  warnings: VaultWarning[];
}

export type ErrorCode =
  | 'VAULT_NOT_CONFIGURED'
  | 'VAULT_NOT_ACCESSIBLE'
  | 'PATH_OUTSIDE_VAULT'
  | 'INVALID_FILE_NAME'
  | 'VAULT_SETUP_FAILED'
  | 'MANIFEST_INVALID';

export interface AppError {
  code: ErrorCode | string;
  message: string;
  recoverable: boolean;
}

export function isAppError(err: unknown): err is AppError {
  if (typeof err === 'object' && err !== null) {
    const candidate = err as Record<string, unknown>;
    return (
      typeof candidate.code === 'string' &&
      typeof candidate.message === 'string' &&
      typeof candidate.recoverable === 'boolean'
    );
  }
  return false;
}

export function formatErrorMessage(err: unknown): string {
  if (isAppError(err)) {
    return err.message;
  }
  if (err instanceof Error) {
    return err.message;
  }
  if (typeof err === 'string') {
    return err;
  }
  return 'Terjadi kesalahan yang tidak terduga';
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

export async function getVaultSettings(): Promise<AppSettingsResponse> {
  return invoke<AppSettingsResponse>('settings_get');
}

export async function validateVault(vaultPath: string): Promise<VaultPreview> {
  return invoke<VaultPreview>('vault_validate', {
    request: { vault_path: vaultPath },
  });
}

export async function setupVault(vaultPath: string): Promise<VaultSetupResult> {
  return invoke<VaultSetupResult>('vault_setup', {
    request: { vault_path: vaultPath },
  });
}

export async function readTasksFromVault(date?: string): Promise<MarkdownTask[]> {
  return invoke<MarkdownTask[]>('read_markdown_tasks', { date });
}

export async function saveTasksToVault(tasks: MarkdownTask[], date?: string): Promise<boolean> {
  return invoke<boolean>('save_markdown_tasks', { tasks, date });
}

export async function appendDailyNote(content: string, date?: string): Promise<boolean> {
  return invoke<boolean>('append_to_markdown', { content, date });
}
