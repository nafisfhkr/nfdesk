import { useCallback, useEffect, useRef, useState } from 'react';
import { Check, Ban, Plus, FolderOpen, Play, ChevronLeft, ChevronRight } from 'lucide-react';
import type { Task } from '../../lib/tasks';
import {
  listTasksForDate,
  createTask,
  completeTask,
  cancelTask,
  previewLegacyTasks,
  todayLocalDate,
} from '../../lib/tasks';
import { getVaultSettings, formatErrorMessage, isAppError } from '../../lib/markdown';
import { shiftLocalDate, formatPlannerDate } from './plannerDate';
import TaskDetailDialog from './TaskDetailDialog';

export type PlannerFocusTask = Pick<Task, 'id' | 'title'>;

export interface PlannerViewProps {
  onStartFocus(task: PlannerFocusTask): void;
}

function priorityLabel(p: Task['priority']): string | null {
  if (p === 'low') return 'Rendah';
  if (p === 'medium') return 'Sedang';
  if (p === 'high') return 'Tinggi';
  return null;
}

function scheduledTimeLabel(iso: string | null): string | null {
  if (!iso) return null;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return null;
  return d.toLocaleTimeString('id-ID', { hour: '2-digit', minute: '2-digit' });
}

function statusMarker(status: Task['status']): { glyph: string; label: string } {
  if (status === 'completed') return { glyph: '✓', label: 'Selesai' };
  if (status === 'cancelled') return { glyph: '✕', label: 'Dibatalkan' };
  if (status === 'in_progress') return { glyph: '◐', label: 'Berjalan' };
  return { glyph: '○', label: 'Rencana' };
}

export default function PlannerView({ onStartFocus }: PlannerViewProps) {
  const [selectedDate, setSelectedDate] = useState(() => todayLocalDate());
  const [tasks, setTasks] = useState<Task[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isVaultConfigured, setIsVaultConfigured] = useState<boolean | null>(null);
  const [migrationPreviewAvailable, setMigrationPreviewAvailable] = useState(false);
  const [migrationPreviewCount, setMigrationPreviewCount] = useState<number | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [quickTitle, setQuickTitle] = useState('');
  const [isAdding, setIsAdding] = useState(false);
  const [busyTaskId, setBusyTaskId] = useState<string | null>(null);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);

  const requestId = useRef(0);

  const loadTasks = useCallback(async (date: string) => {
    const current = ++requestId.current;
    setIsLoading(true);
    setError(null);
    try {
      const settings = await getVaultSettings();
      if (current !== requestId.current) return;
      setIsVaultConfigured(settings.vault_configured);
      if (!settings.vault_configured) {
        setTasks([]);
        setMigrationPreviewAvailable(false);
        return;
      }
      const response = await listTasksForDate(date);
      if (current !== requestId.current) return;
      setTasks(response.tasks);
      setMigrationPreviewAvailable(response.migration_preview_available);
    } catch (cause) {
      if (current !== requestId.current) return;
      if (isAppError(cause) && cause.code === 'VAULT_NOT_CONFIGURED') {
        setIsVaultConfigured(false);
        setTasks([]);
      } else {
        setError(formatErrorMessage(cause));
      }
    } finally {
      if (current === requestId.current) setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTasks(selectedDate);
  }, [selectedDate, loadTasks]);

  const handleQuickAdd = async () => {
    const trimmed = quickTitle.trim();
    if (!trimmed || isAdding || migrationPreviewAvailable) return;
    setIsAdding(true);
    setError(null);
    try {
      const created = await createTask({ title: trimmed, planned_date: selectedDate });
      setTasks((prev) => [...prev, created]);
      setQuickTitle('');
    } catch (cause) {
      setError(formatErrorMessage(cause));
    } finally {
      setIsAdding(false);
    }
  };

  const handleQuickKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.nativeEvent.isComposing && quickTitle.trim() !== '' && !isAdding) {
      e.preventDefault();
      handleQuickAdd();
    }
  };

  const handleComplete = async (task: Task) => {
    if (task.status === 'completed' || task.status === 'cancelled' || busyTaskId) return;
    setBusyTaskId(task.id);
    setError(null);
    try {
      const updated = await completeTask(task.id, task.planned_date);
      setTasks((prev) => prev.map((t) => (t.id === task.id ? updated : t)));
    } catch (cause) {
      setError(formatErrorMessage(cause));
    } finally {
      setBusyTaskId(null);
    }
  };

  const handleCancel = async (task: Task) => {
    if (task.status === 'cancelled' || busyTaskId) return;
    setBusyTaskId(task.id);
    setError(null);
    try {
      const updated = await cancelTask(task.id, task.planned_date);
      setTasks((prev) => prev.map((t) => (t.id === task.id ? updated : t)));
    } catch (cause) {
      setError(formatErrorMessage(cause));
    } finally {
      setBusyTaskId(null);
    }
  };

  const handlePreviewLegacy = async () => {
    try {
      setPreviewLoading(true);
      setError(null);
      const preview = await previewLegacyTasks(selectedDate);
      setMigrationPreviewCount(preview.items.length);
    } catch (cause) {
      setError(formatErrorMessage(cause));
    } finally {
      setPreviewLoading(false);
    }
  };

  if (isVaultConfigured === false) {
    return (
      <section
        aria-label="Planner"
        className="flex flex-col h-full items-center justify-center text-center px-6 rounded-xl bg-[#faf6ec] text-stone-700 border border-stone-900/10"
      >
        <FolderOpen className="w-10 h-10 mb-3 text-stone-400" aria-hidden="true" />
        <p className="text-sm">
          Obsidian Vault belum terhubung. Silakan buka Settings dan pilih Vault untuk mengaktifkan sinkronisasi task.
        </p>
      </section>
    );
  }

  const actionable = tasks.filter((t) => t.status === 'planned' || t.status === 'in_progress' || t.status === 'completed');
  const completed = tasks.filter((t) => t.status === 'completed');
  const cancelled = tasks.filter((t) => t.status === 'cancelled');
  const percent = actionable.length === 0 ? 0 : Math.round((completed.length / actionable.length) * 100);
  const isToday = selectedDate === todayLocalDate();

  return (
    <section
      aria-label="Planner"
      className="flex flex-col h-full overflow-hidden rounded-xl bg-[#faf6ec] text-stone-800 border border-stone-900/10"
    >
      <div
        className="flex-1 overflow-y-auto px-3 py-3 pb-4 custom-scrollbar"
        style={{
          backgroundImage: 'repeating-linear-gradient(to bottom, transparent 0, transparent 27px, rgba(120,100,70,0.08) 27px, rgba(120,100,70,0.08) 28px)',
        }}
      >
        {/* Date header */}
        <div className="mb-2 flex items-center justify-between gap-2 border-b border-stone-900/15 pb-2">
          <button
            type="button"
            aria-label="Hari sebelumnya"
            onClick={() => setSelectedDate((d) => shiftLocalDate(d, -1))}
            className="p-1.5 rounded-lg text-stone-600 hover:text-stone-900 hover:bg-stone-900/5 border border-stone-900/15 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800"
          >
            <ChevronLeft className="w-4 h-4" aria-hidden="true" />
          </button>
          <div className="flex-1 text-center min-w-0">
            <h1 className="text-sm font-bold font-mono text-stone-900 truncate">{formatPlannerDate(selectedDate)}</h1>
            <p className="text-[10px] font-mono text-stone-500">{selectedDate}</p>
          </div>
          <button
            type="button"
            aria-label="Hari berikutnya"
            onClick={() => setSelectedDate((d) => shiftLocalDate(d, 1))}
            className="p-1.5 rounded-lg text-stone-600 hover:text-stone-900 hover:bg-stone-900/5 border border-stone-900/15 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800"
          >
            <ChevronRight className="w-4 h-4" aria-hidden="true" />
          </button>
          <button
            type="button"
            aria-label="Kembali ke hari ini"
            disabled={isToday}
            onClick={() => setSelectedDate(todayLocalDate())}
            className="px-2.5 py-1.5 rounded-lg text-[11px] font-bold font-mono border transition-colors disabled:opacity-40 disabled:cursor-default bg-white text-stone-700 border-stone-900/20 hover:bg-stone-900/5 focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800"
          >
            Hari ini
          </button>
        </div>

        {/* Progress */}
        <div className="mb-3 flex items-center gap-2" aria-live="polite">
          <span className="text-[11px] font-mono text-stone-600 whitespace-nowrap">
            {completed.length}/{actionable.length} selesai · {percent}%
            {cancelled.length > 0 && <span> · {cancelled.length} dibatalkan</span>}
          </span>
          <div
            className="flex-1 h-1.5 rounded-full bg-stone-900/10 overflow-hidden"
            role="progressbar"
            aria-valuenow={percent}
            aria-valuemin={0}
            aria-valuemax={100}
          >
            <div className="h-full bg-stone-800 transition-all" style={{ width: `${percent}%` }} />
          </div>
        </div>

        {/* Migration banner */}
        {migrationPreviewAvailable && (
          <div className="mb-3 p-3 bg-amber-500/15 border border-amber-700/30 rounded-xl text-xs text-amber-900 flex flex-col gap-2">
            <div>File task lama terdeteksi. Tambah atau ubah task dinonaktifkan sampai migrasi formal tersedia.</div>
            <div className="flex items-center gap-2">
              <button
                type="button"
                aria-label="Pratinjau task lama"
                onClick={handlePreviewLegacy}
                disabled={previewLoading}
                className="px-2.5 py-1 bg-amber-700/15 hover:bg-amber-700/25 text-amber-900 border border-amber-700/30 rounded-lg text-[11px] font-bold transition-colors disabled:opacity-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-amber-800"
              >
                {previewLoading ? 'Memeriksa...' : 'Pratinjau Checklist Lama'}
              </button>
              {migrationPreviewCount !== null && (
                <span className="text-amber-900/80 text-[11px]">{migrationPreviewCount} item checklist ditemukan.</span>
              )}
            </div>
          </div>
        )}

        {/* Error banner */}
        {error && (
          <div className="mb-3 text-xs text-red-800 bg-red-500/10 border border-red-700/30 px-3 py-2 rounded-xl flex items-center justify-between gap-2">
            <span>{error}</span>
            <button
              type="button"
              aria-label="Coba lagi memuat task"
              onClick={() => loadTasks(selectedDate)}
              className="px-2 py-0.5 bg-red-700/10 hover:bg-red-700/20 text-red-800 rounded border border-red-700/30 text-[10px] font-bold transition-colors flex-shrink-0 focus:outline-none focus-visible:ring-2 focus-visible:ring-red-800"
            >
              Coba Lagi
            </button>
          </div>
        )}

        {/* Quick add */}
        <div className="relative mb-3">
          <input
            type="text"
            value={quickTitle}
            onChange={(e) => setQuickTitle(e.target.value)}
            onKeyDown={handleQuickKeyDown}
            placeholder={migrationPreviewAvailable ? 'Migrasi diperlukan sebelum menambah task' : 'Ketik task baru... (Enter)'}
            disabled={isAdding || migrationPreviewAvailable}
            aria-label="Judul task baru"
            className="w-full bg-white border border-stone-900/20 rounded-xl py-2.5 pl-4 pr-20 text-sm font-mono text-stone-900 placeholder:text-stone-400 placeholder:font-sans focus:outline-none focus:border-stone-800 focus:ring-2 focus:ring-stone-800/20 transition-all disabled:opacity-50"
          />
          <button
            type="button"
            aria-label="Tambah task"
            onClick={handleQuickAdd}
            disabled={isAdding || migrationPreviewAvailable || quickTitle.trim() === ''}
            className="absolute right-2 top-1/2 -translate-y-1/2 px-2.5 py-1 text-[11px] font-bold text-amber-50 bg-stone-900 hover:bg-stone-700 rounded-lg transition-colors disabled:opacity-50 flex items-center gap-1 focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800 focus-visible:ring-offset-2 focus-visible:ring-offset-[#faf6ec]"
          >
            <Plus className="w-3.5 h-3.5" aria-hidden="true" />
            {isAdding ? 'Menambah…' : 'Tambah'}
          </button>
        </div>

        {/* List states */}
        {isLoading && tasks.length === 0 ? (
          <div className="text-center py-8 text-stone-500 text-sm font-mono">Memuat task…</div>
        ) : !isLoading && tasks.length === 0 && !error ? (
          <div className="text-center py-8 text-stone-500 text-sm border border-dashed border-stone-900/20 rounded-xl bg-white/50">
            <p className="font-mono font-bold text-stone-700">Belum ada task untuk hari ini.</p>
            <p className="mt-1 text-xs">Tambahkan task pertama dengan quick add di atas.</p>
          </div>
        ) : (
          <div className="space-y-1.5">
            {tasks.map((task) => {
              const priority = priorityLabel(task.priority);
              const scheduled = scheduledTimeLabel(task.scheduled_at);
              const marker = statusMarker(task.status);
              const isBusy = busyTaskId === task.id;
              const isCancelled = task.status === 'cancelled';
              const isDone = task.status === 'completed';
              return (
                <div
                  key={task.id}
                  role="button"
                  tabIndex={0}
                  aria-label={`Buka detail: ${task.title}`}
                  onClick={() => setSelectedTask(task)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && e.target === e.currentTarget) setSelectedTask(task);
                  }}
                  className={`group flex items-center gap-2.5 px-3 py-2.5 bg-white/80 border border-stone-900/10 border-l-4 hover:border-stone-900/25 hover:bg-white rounded-lg transition-all cursor-pointer focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800 ${
                    isDone ? 'border-l-emerald-700' : isCancelled ? 'border-l-stone-400 opacity-70' : 'border-l-stone-800'
                  }`}
                >
                  <button
                    type="button"
                    aria-label={isDone ? `Task selesai: ${task.title}` : `Selesaikan task: ${task.title}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleComplete(task);
                    }}
                    disabled={isDone || isCancelled || isBusy || migrationPreviewAvailable}
                    className={`flex-shrink-0 w-5 h-5 rounded border flex items-center justify-center transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800 ${
                      isDone
                        ? 'bg-emerald-700 border-emerald-700 cursor-default'
                        : isCancelled
                          ? 'bg-stone-200 border-stone-300 text-stone-500 cursor-default'
                          : 'border-stone-500 hover:border-stone-800 bg-white disabled:opacity-50'
                    }`}
                  >
                    {isDone && <Check className="w-3.5 h-3.5 text-white" aria-hidden="true" />}
                    {isCancelled && <Ban className="w-3 h-3 text-stone-500" aria-hidden="true" />}
                  </button>

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 min-w-0">
                      <span
                        className={`text-sm truncate font-medium ${isDone || isCancelled ? 'text-stone-400 line-through' : 'text-stone-900'}`}
                      >
                        {task.title}
                      </span>
                    </div>
                    <div className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-[10px] font-mono text-stone-500">
                      <span className="font-bold text-stone-700">
                        {marker.glyph} {marker.label}
                      </span>
                      {priority && (
                        <span className="px-1.5 py-px rounded border border-stone-900/15 bg-stone-900/5 text-stone-700">
                          {priority}
                        </span>
                      )}
                      {scheduled && <span>⏰ {scheduled}</span>}
                      {task.estimated_sessions != null && (
                        <span>
                          ≈ {task.estimated_sessions} sesi
                          {task.estimated_session_minutes != null ? ` × ${task.estimated_session_minutes}m` : ''}
                        </span>
                      )}
                      {isBusy && <span className="text-stone-800 font-bold">Memproses…</span>}
                    </div>
                  </div>

                  {!isDone && !isCancelled && (
                    <button
                      type="button"
                      aria-label={`Mulai fokus: ${task.title}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        onStartFocus({ id: task.id, title: task.title });
                      }}
                      className="p-1.5 rounded-lg text-stone-600 hover:text-white hover:bg-stone-900 border border-stone-900/15 transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-stone-800"
                    >
                      <Play className="w-3.5 h-3.5 fill-current" aria-hidden="true" />
                    </button>
                  )}
                  {!isCancelled && (
                    <button
                      type="button"
                      aria-label={`Batalkan task: ${task.title}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleCancel(task);
                      }}
                      disabled={isBusy || migrationPreviewAvailable}
                      className="p-1.5 rounded-lg text-stone-500 hover:text-red-700 hover:bg-red-500/10 border border-transparent hover:border-red-700/30 transition-all disabled:opacity-50 focus:outline-none focus-visible:ring-2 focus-visible:ring-red-800"
                    >
                      <Ban className="w-3.5 h-3.5" aria-hidden="true" />
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
      {selectedTask && (
        <TaskDetailDialog
          task={selectedTask}
          isBusy={busyTaskId !== null}
          onClose={() => {
            if (busyTaskId === null) setSelectedTask(null);
          }}
          onSaved={(updated) => {
            setTasks((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
            setSelectedTask(updated);
          }}
          onRescheduled={() => {
            setSelectedTask(null);
            void loadTasks(selectedDate);
          }}
          onCancelled={(updated) => {
            setTasks((prev) => prev.map((t) => (t.id === updated.id ? updated : t)));
            setSelectedTask(updated);
          }}
          onStartFocus={(focus) => {
            setSelectedTask(null);
            onStartFocus(focus);
          }}
        />
      )}
    </section>
  );
}
