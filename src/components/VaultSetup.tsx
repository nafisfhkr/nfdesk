import { useState, useEffect, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, AlertTriangle, CheckCircle2, RefreshCw } from 'lucide-react';
import {
  getVaultSettings,
  validateVault,
  setupVault,
  isAppError,
  formatErrorMessage,
  type VaultPreview,
  type VaultSetupResult,
  type AppError,
} from '../lib/markdown';

export type VaultSetupState =
  | { kind: 'idle' }
  | { kind: 'picking' }
  | { kind: 'validating' }
  | { kind: 'preview'; selectedPath: string; preview: VaultPreview }
  | { kind: 'setting-up'; selectedPath: string; preview: VaultPreview }
  | { kind: 'success'; result: VaultSetupResult }
  | { kind: 'error'; error: AppError; selectedPath?: string };

export interface VaultSetupProps {
  onVaultConfigured?: () => void;
}

export default function VaultSetup({ onVaultConfigured }: VaultSetupProps) {
  const [state, setState] = useState<VaultSetupState>({ kind: 'idle' });
  const [currentVaultPath, setCurrentVaultPath] = useState<string | null>(null);
  const [isLoadingInitial, setIsLoadingInitial] = useState(true);

  const checkInitialSettings = useCallback(async () => {
    try {
      const settings = await getVaultSettings();
      if (settings.vault_configured && settings.vault_path) {
        setCurrentVaultPath(settings.vault_path);
      } else {
        setCurrentVaultPath(null);
      }
    } catch {
      setCurrentVaultPath(null);
    } finally {
      setIsLoadingInitial(false);
    }
  }, []);

  useEffect(() => {
    checkInitialSettings();
  }, [checkInitialSettings]);

  const handlePickFolder = async () => {
    setState({ kind: 'picking' });
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Pilih Obsidian Vault',
      });

      if (!selected) {
        setState({ kind: 'idle' });
        return;
      }

      const selectedPath = typeof selected === 'string' ? selected : selected[0];
      if (!selectedPath) {
        setState({ kind: 'idle' });
        return;
      }

      setState({ kind: 'validating' });
      const preview = await validateVault(selectedPath);
      setState({ kind: 'preview', selectedPath, preview });
    } catch (err) {
      const appErr: AppError = isAppError(err)
        ? err
        : {
            code: 'VAULT_SETUP_FAILED',
            message: formatErrorMessage(err),
            recoverable: true,
          };
      setState({ kind: 'error', error: appErr });
    }
  };

  const handleConfirmSetup = async (selectedPath: string, preview: VaultPreview) => {
    setState({ kind: 'setting-up', selectedPath, preview });
    try {
      const result = await setupVault(selectedPath);
      setState({ kind: 'success', result });
      setCurrentVaultPath(result.vault_path);
      onVaultConfigured?.();
    } catch (err) {
      const appErr: AppError = isAppError(err)
        ? err
        : {
            code: 'VAULT_SETUP_FAILED',
            message: formatErrorMessage(err),
            recoverable: true,
          };
      setState({ kind: 'error', error: appErr, selectedPath });
    }
  };

  const handleRetry = (selectedPath?: string) => {
    if (selectedPath) {
      setState({ kind: 'validating' });
      validateVault(selectedPath)
        .then((preview) => {
          setState({ kind: 'preview', selectedPath, preview });
        })
        .catch((err) => {
          const appErr: AppError = isAppError(err)
            ? err
            : {
                code: 'VAULT_SETUP_FAILED',
                message: formatErrorMessage(err),
                recoverable: true,
              };
          setState({ kind: 'error', error: appErr, selectedPath });
        });
    } else {
      handlePickFolder();
    }
  };

  if (isLoadingInitial) {
    return (
      <div className="p-3 bg-white/5 border border-white/10 rounded-xl text-xs text-slate-400 text-center">
        Memeriksa status Vault…
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {/* Current vault status */}
      <div className="p-3 bg-white/5 border border-white/10 rounded-xl">
        <div className="flex items-center justify-between gap-2">
          <div className="min-w-0">
            <span className="text-[10px] uppercase font-bold tracking-wider text-slate-400 block mb-1">
              Status Vault
            </span>
            {currentVaultPath ? (
              <div className="text-xs text-emerald-400 flex items-center gap-1.5 font-medium truncate">
                <CheckCircle2 className="w-3.5 h-3.5 flex-shrink-0" />
                <span className="truncate" title={currentVaultPath}>
                  {currentVaultPath}
                </span>
              </div>
            ) : (
              <div className="text-xs text-amber-400/90 flex items-center gap-1.5 font-medium">
                <AlertTriangle className="w-3.5 h-3.5 flex-shrink-0" />
                <span>Belum disiapkan</span>
              </div>
            )}
          </div>
          <button
            onClick={handlePickFolder}
            disabled={state.kind === 'validating' || state.kind === 'setting-up'}
            className="flex-shrink-0 flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-indigo-600/30 hover:bg-indigo-600/50 border border-indigo-500/40 text-xs font-semibold text-indigo-200 transition-all active:scale-95 disabled:opacity-50"
          >
            <FolderOpen className="w-3.5 h-3.5" />
            {currentVaultPath ? 'Ganti' : 'Pilih Folder'}
          </button>
        </div>
      </div>

      {/* Validating indicator */}
      {state.kind === 'validating' && (
        <div className="p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-xl text-xs text-indigo-300 flex items-center gap-2">
          <RefreshCw className="w-3.5 h-3.5 animate-spin" />
          <span>Memvalidasi direktori Vault…</span>
        </div>
      )}

      {/* Preview and Warnings */}
      {state.kind === 'preview' && (
        <div className="p-3 bg-white/5 border border-white/10 rounded-xl space-y-3">
          <div>
            <span className="text-[10px] uppercase font-bold tracking-wider text-slate-400 block mb-1">
              Path Terpilih
            </span>
            <p className="text-xs text-slate-200 break-all font-mono bg-black/20 p-2 rounded-lg">
              {state.selectedPath}
            </p>
          </div>

          {/* Directory creation plan */}
          <div className="text-xs space-y-1">
            <span className="text-[10px] uppercase font-bold tracking-wider text-slate-400 block mb-1">
              Struktur Folder NFDesk
            </span>
            {state.preview.directories_to_create.length > 0 && (
              <div className="text-indigo-300 text-[11px]">
                Akan dibuat: {state.preview.directories_to_create.join(', ')}
              </div>
            )}
            {state.preview.existing_directories.length > 0 && (
              <div className="text-slate-400 text-[11px]">
                Sudah ada: {state.preview.existing_directories.join(', ')}
              </div>
            )}
          </div>

          {/* Warnings */}
          {state.preview.warnings.length > 0 && (
            <div className="space-y-1.5 pt-1">
              {state.preview.warnings.map((w, idx) => (
                <div
                  key={idx}
                  className="p-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-300 text-xs flex items-start gap-2"
                >
                  <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5 text-amber-400" />
                  <span className="leading-snug">{w.message}</span>
                </div>
              ))}
            </div>
          )}

          {/* Action buttons */}
          <div className="flex justify-end gap-2 pt-1">
            <button
              onClick={() => setState({ kind: 'idle' })}
              className="px-3 py-1.5 rounded-lg text-xs font-semibold text-slate-400 border border-white/10 hover:bg-white/5 transition-all"
            >
              Batal
            </button>
            <button
              onClick={() => handleConfirmSetup(state.selectedPath, state.preview)}
              className="px-3.5 py-1.5 rounded-lg text-xs font-bold bg-indigo-600 hover:bg-indigo-500 text-white shadow-md shadow-indigo-600/30 active:scale-95 transition-all"
            >
              Siapkan NFDesk
            </button>
          </div>
        </div>
      )}

      {/* Setting up indicator */}
      {state.kind === 'setting-up' && (
        <div className="p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-xl text-xs text-indigo-300 flex items-center gap-2">
          <RefreshCw className="w-3.5 h-3.5 animate-spin" />
          <span>Menyiapkan Vault…</span>
        </div>
      )}

      {/* Success notification */}
      {state.kind === 'success' && (
        <div className="p-3 bg-emerald-500/10 border border-emerald-500/20 rounded-xl text-xs text-emerald-300 space-y-1">
          <div className="flex items-center gap-1.5 font-semibold text-emerald-400">
            <CheckCircle2 className="w-4 h-4" />
            <span>Vault berhasil disiapkan!</span>
          </div>
          <p className="text-[11px] text-emerald-300/80">
            {state.result.manifest_created
              ? 'Manifest schema v1 dan folder skeleton telah dibuat.'
              : 'Struktur NFDesk telah tervalidasi dan siap digunakan.'}
          </p>
        </div>
      )}

      {/* Error notification & recovery */}
      {state.kind === 'error' && (
        <div className="p-3 bg-rose-500/10 border border-rose-500/20 rounded-xl text-xs text-rose-300 space-y-2">
          <div className="flex items-start gap-2">
            <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5 text-rose-400" />
            <div className="space-y-0.5">
              <span className="font-semibold block">{state.error.message}</span>
              <span className="text-[10px] text-rose-400/80 font-mono">
                Code: {state.error.code}
              </span>
            </div>
          </div>
          <div className="flex justify-end gap-2 pt-1">
            {state.error.recoverable && (
              <button
                onClick={() => handleRetry(state.selectedPath)}
                className="px-2.5 py-1 rounded-lg text-xs font-semibold bg-rose-500/20 hover:bg-rose-500/30 text-rose-200 border border-rose-500/30"
              >
                Coba Lagi
              </button>
            )}
            <button
              onClick={handlePickFolder}
              className="px-2.5 py-1 rounded-lg text-xs font-semibold bg-white/10 hover:bg-white/15 text-slate-200"
            >
              Pilih Vault Lain
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
