import { useEffect, useRef, useState } from 'react';
import type { Task, TaskPriority } from '../../lib/tasks';
import {
  updateTask,
  rescheduleTask,
  carryTaskToToday,
  completeTask,
  cancelTask,
  todayLocalDate,
} from '../../lib/tasks';
import { formatErrorMessage } from '../../lib/markdown';
import type { PlannerFocusTask } from './PlannerView';

interface TaskDetailDialogProps {
  task: Task;
  isBusy: boolean;
  onClose(): void;
  onSaved(task: Task): void;
  onRescheduled(task: Task): void;
  onCancelled(task: Task): void;
  onStartFocus(task: PlannerFocusTask): void;
}

function pad2(n: number): string {
  return String(n).padStart(2, '0');
}

function toDateTimeLocal(iso: string | null): string {
  if (!iso) return '';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}T${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

function fromDateTimeLocal(value: string): string | null {
  if (!value) return null;
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return null;
  return d.toISOString();
}

function isPositiveIntString(v: string): boolean {
  if (v === '') return false;
  if (!/^\d+$/.test(v)) return false;
  return Number(v) > 0;
}

export default function TaskDetailDialog({
  task,
  isBusy,
  onClose,
  onSaved,
  onRescheduled,
  onCancelled,
  onStartFocus,
}: TaskDetailDialogProps) {
  const [title, setTitle] = useState(task.title);
  const [description, setDescription] = useState(task.description ?? '');
  const [priority, setPriority] = useState<string>(task.priority ?? '');
  const [scheduledAt, setScheduledAt] = useState(() => toDateTimeLocal(task.scheduled_at));
  const [deadlineAt, setDeadlineAt] = useState(() => toDateTimeLocal(task.deadline_at));
  const [sessions, setSessions] = useState(task.estimated_sessions != null ? String(task.estimated_sessions) : '');
  const [minutes, setMinutes] = useState(
    task.estimated_session_minutes != null ? String(task.estimated_session_minutes) : '',
  );
  const [rescheduleDate, setRescheduleDate] = useState('');
  const [formError, setFormError] = useState<string | null>(null);
  const [internalBusy, setInternalBusy] = useState(false);
  const titleRef = useRef<HTMLInputElement>(null);
  const busy = isBusy || internalBusy;

  useEffect(() => {
    setTitle(task.title);
    setDescription(task.description ?? '');
    setPriority(task.priority ?? '');
    setScheduledAt(toDateTimeLocal(task.scheduled_at));
    setDeadlineAt(toDateTimeLocal(task.deadline_at));
    setSessions(task.estimated_sessions != null ? String(task.estimated_sessions) : '');
    setMinutes(task.estimated_session_minutes != null ? String(task.estimated_session_minutes) : '');
    setRescheduleDate('');
    setFormError(null);
  }, [task]);

  useEffect(() => {
    titleRef.current?.focus();
  }, []);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !busy) {
        e.preventDefault();
        e.stopPropagation();
        onClose();
      }
    };
    window.addEventListener('keydown', handleKey, true);
    return () => window.removeEventListener('keydown', handleKey, true);
  }, [onClose, busy]);

  const canStartFocus = task.status === 'planned' || task.status === 'in_progress';

  const validateEstimate = (): string | null => {
    const sEmpty = sessions.trim() === '';
    const mEmpty = minutes.trim() === '';
    if (sEmpty && mEmpty) return null;
    if (sEmpty !== mEmpty) return 'Isi kedua field estimasi, atau kosongkan keduanya.';
    if (!isPositiveIntString(sessions.trim()) || !isPositiveIntString(minutes.trim())) {
      return 'Estimasi harus bilangan bulat lebih dari 0.';
    }
    return null;
  };

  const handleSave = async () => {
    if (busy) return;
    const trimmedTitle = title.trim();
    if (!trimmedTitle) {
      setFormError('Judul task tidak boleh kosong.');
      return;
    }
    const estimateError = validateEstimate();
    if (estimateError) {
      setFormError(estimateError);
      return;
    }
    setFormError(null);
    setInternalBusy(true);
    try {
      const updated = await updateTask(task.id, task.planned_date, {
        title: trimmedTitle,
        description: description.trim() === '' ? null : description,
        priority: (priority === '' ? null : priority) as TaskPriority | null,
        scheduled_at: fromDateTimeLocal(scheduledAt),
        deadline_at: fromDateTimeLocal(deadlineAt),
        estimated_sessions: sessions.trim() === '' ? null : Number(sessions.trim()),
        estimated_session_minutes: minutes.trim() === '' ? null : Number(minutes.trim()),
      });
      onSaved(updated);
    } catch (cause) {
      setFormError(formatErrorMessage(cause));
    } finally {
      setInternalBusy(false);
    }
  };

  const handleReschedule = async () => {
    if (busy || !rescheduleDate || rescheduleDate === task.planned_date) return;
    setFormError(null);
    setInternalBusy(true);
    try {
      const moved = await rescheduleTask(task.id, task.planned_date, rescheduleDate);
      onRescheduled(moved);
    } catch (cause) {
      setFormError(formatErrorMessage(cause));
    } finally {
      setInternalBusy(false);
    }
  };

  const handleCarry = async () => {
    if (busy || task.planned_date === todayLocalDate()) return;
    setFormError(null);
    setInternalBusy(true);
    try {
      const moved = await carryTaskToToday(task.id, task.planned_date);
      onRescheduled(moved);
    } catch (cause) {
      setFormError(formatErrorMessage(cause));
    } finally {
      setInternalBusy(false);
    }
  };

  const handleComplete = async () => {
    if (busy) return;
    setFormError(null);
    setInternalBusy(true);
    try {
      const updated = await completeTask(task.id, task.planned_date);
      onSaved(updated);
    } catch (cause) {
      setFormError(formatErrorMessage(cause));
    } finally {
      setInternalBusy(false);
    }
  };

  const handleCancelTask = async () => {
    if (busy) return;
    if (!window.confirm('Batalkan task ini? Task tidak akan dihapus.')) return;
    setFormError(null);
    setInternalBusy(true);
    try {
      const updated = await cancelTask(task.id, task.planned_date);
      onCancelled(updated);
    } catch (cause) {
      setFormError(formatErrorMessage(cause));
    } finally {
      setInternalBusy(false);
    }
  };

  const handleFormKeyDown = (e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter' && !busy) {
      e.preventDefault();
      handleSave();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
      onClick={() => {
        if (!busy) onClose();
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="task-detail-title"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleFormKeyDown}
        className="w-full max-w-sm max-h-[90vh] overflow-y-auto custom-scrollbar rounded-2xl border border-amber-900/20 bg-[#faf6ec] text-stone-800 p-5 shadow-2xl"
      >
        <div className="flex items-center justify-between mb-3">
          <h2 id="task-detail-title" className="text-sm font-bold font-mono">
            Detail Task
          </h2>
          <button
            type="button"
            aria-label="Tutup detail task"
            onClick={onClose}
            disabled={busy}
            className="px-2 py-1 rounded-lg text-xs text-stone-500 hover:text-stone-800 hover:bg-stone-900/5 border border-transparent hover:border-stone-900/10 disabled:opacity-50"
          >
            ✕ Tutup
          </button>
        </div>

        <p className="text-[11px] font-mono text-stone-500 mb-3">Planned date: {task.planned_date} (read-only)</p>

        {formError && (
          <div className="mb-3 text-xs text-red-700 bg-red-500/10 border border-red-500/30 px-3 py-2 rounded-xl">
            {formError}
          </div>
        )}

        <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-name">
          Judul
        </label>
        <input
          id="task-detail-name"
          ref={titleRef}
          type="text"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          disabled={busy}
          className="w-full text-sm py-2 px-3 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 transition-colors disabled:opacity-50"
        />

        <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mt-3 mb-1" htmlFor="task-detail-desc">
          Deskripsi
        </label>
        <textarea
          id="task-detail-desc"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          disabled={busy}
          rows={3}
          placeholder="Deskripsi opsional..."
          className="w-full text-sm py-2 px-3 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 transition-colors resize-y disabled:opacity-50"
        />

        <div className="grid grid-cols-2 gap-2 mt-3">
          <div>
            <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-priority">
              Prioritas
            </label>
            <select
              id="task-detail-priority"
              value={priority}
              onChange={(e) => setPriority(e.target.value)}
              disabled={busy}
              className="w-full text-sm py-2 px-2 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 disabled:opacity-50"
            >
              <option value="">Tanpa prioritas</option>
              <option value="low">Rendah</option>
              <option value="medium">Sedang</option>
              <option value="high">Tinggi</option>
            </select>
          </div>
          <div>
            <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-reschedule">
              Jadwalkan ulang
            </label>
            <input
              id="task-detail-reschedule"
              type="date"
              value={rescheduleDate}
              onChange={(e) => setRescheduleDate(e.target.value)}
              disabled={busy}
              className="w-full text-sm py-2 px-2 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 disabled:opacity-50"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2 mt-2">
          <div>
            <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-scheduled">
              Terjadwal
            </label>
            <input
              id="task-detail-scheduled"
              type="datetime-local"
              value={scheduledAt}
              onChange={(e) => setScheduledAt(e.target.value)}
              disabled={busy}
              className="w-full text-xs py-2 px-2 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 disabled:opacity-50"
            />
          </div>
          <div>
            <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-deadline">
              Deadline
            </label>
            <input
              id="task-detail-deadline"
              type="datetime-local"
              value={deadlineAt}
              onChange={(e) => setDeadlineAt(e.target.value)}
              disabled={busy}
              className="w-full text-xs py-2 px-2 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 disabled:opacity-50"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-2 mt-2">
          <div>
            <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-sessions">
              Estimasi sesi
            </label>
            <input
              id="task-detail-sessions"
              type="number"
              min={1}
              step={1}
              value={sessions}
              onChange={(e) => setSessions(e.target.value)}
              disabled={busy}
              placeholder="cth. 2"
              className="w-full text-sm py-2 px-3 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 disabled:opacity-50"
            />
          </div>
          <div>
            <label className="block text-[11px] font-semibold uppercase tracking-wider text-stone-500 mb-1" htmlFor="task-detail-minutes">
              Menit / sesi
            </label>
            <input
              id="task-detail-minutes"
              type="number"
              min={1}
              step={1}
              value={minutes}
              onChange={(e) => setMinutes(e.target.value)}
              disabled={busy}
              placeholder="cth. 25"
              className="w-full text-sm py-2 px-3 rounded-xl bg-white border border-stone-900/15 outline-none focus:border-stone-800 disabled:opacity-50"
            />
          </div>
        </div>

        <button
          type="button"
          onClick={handleSave}
          disabled={busy}
          className="w-full mt-4 py-2.5 rounded-xl text-sm font-bold bg-stone-900 text-amber-50 hover:bg-stone-800 active:scale-[0.99] transition-all disabled:opacity-50"
        >
          {internalBusy ? 'Menyimpan…' : 'Simpan Perubahan'}
        </button>

        <div className="grid grid-cols-2 gap-2 mt-2">
          <button
            type="button"
            onClick={handleReschedule}
            disabled={busy || !rescheduleDate || rescheduleDate === task.planned_date}
            className="py-2 rounded-xl text-xs font-bold border border-stone-900/20 text-stone-700 hover:bg-stone-900/5 transition-all disabled:opacity-40"
          >
            Jadwalkan ulang
          </button>
          {task.planned_date !== todayLocalDate() ? (
            <button
              type="button"
              onClick={handleCarry}
              disabled={busy}
              className="py-2 rounded-xl text-xs font-bold border border-stone-900/20 text-stone-700 hover:bg-stone-900/5 transition-all disabled:opacity-40"
            >
              Bawa ke hari ini
            </button>
          ) : (
            <button
              type="button"
              onClick={handleComplete}
              disabled={busy || task.status === 'completed' || task.status === 'cancelled'}
              className="py-2 rounded-xl text-xs font-bold bg-emerald-700 text-white hover:bg-emerald-600 transition-all disabled:opacity-40"
            >
              Selesaikan
            </button>
          )}
        </div>

        {task.planned_date !== todayLocalDate() && (
          <button
            type="button"
            onClick={handleComplete}
            disabled={busy || task.status === 'completed' || task.status === 'cancelled'}
            className="w-full mt-2 py-2 rounded-xl text-xs font-bold bg-emerald-700 text-white hover:bg-emerald-600 transition-all disabled:opacity-40"
          >
            Selesaikan
          </button>
        )}

        <div className="grid grid-cols-2 gap-2 mt-2">
          <button
            type="button"
            onClick={handleCancelTask}
            disabled={busy || task.status === 'cancelled'}
            className="py-2 rounded-xl text-xs font-bold border border-red-700/30 text-red-700 hover:bg-red-500/10 transition-all disabled:opacity-40"
          >
            Batalkan Task
          </button>
          {canStartFocus && (
            <button
              type="button"
              onClick={() => onStartFocus({ id: task.id, title: task.title })}
              disabled={busy}
              className="py-2 rounded-xl text-xs font-bold bg-indigo-600 text-white hover:bg-indigo-500 transition-all disabled:opacity-40"
            >
              ▶ Mulai Fokus
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
