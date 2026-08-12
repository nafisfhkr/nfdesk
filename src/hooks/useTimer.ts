import { useCallback, useEffect, useRef, useState } from 'react';

export type TimerMode = 'FOCUS' | 'BREAK';
export type TimerStatus = 'IDLE' | 'RUNNING' | 'PAUSED' | 'COMPLETED';

export const DURATION_KEYS = {
  FOCUS: 'nfdesk-focus-time',
  BREAK: 'nfdesk-break-time',
} as const;

export const DEFAULT_DURATION_MIN: Record<TimerMode, number> = {
  FOCUS: 25,
  BREAK: 5,
};

export function getDurationMin(mode: TimerMode): number {
  const raw = localStorage.getItem(DURATION_KEYS[mode]);
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : DEFAULT_DURATION_MIN[mode];
}

export function getDurationMs(mode: TimerMode): number {
  return getDurationMin(mode) * 60 * 1000;
}

export function formatTime(ms: number): string {
  const totalSec = Math.max(0, Math.ceil(ms / 1000));
  const m = Math.floor(totalSec / 60);
  const s = totalSec % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/**
 * Timestamp-based timer to avoid drift when the system sleeps or lags.
 * remaining = baseMs - (now - startedAt). The interval only repaints the UI.
 */
export function useTimer() {
  const [mode, setMode] = useState<TimerMode>('FOCUS');
  const [status, setStatus] = useState<TimerStatus>('IDLE');
  const [remaining, setRemaining] = useState(() => getDurationMs('FOCUS'));

  // remaining at the moment of the last start/resume
  const baseRef = useRef(getDurationMs('FOCUS'));
  const startedAtRef = useRef(0);

  useEffect(() => {
    if (status !== 'RUNNING') return;
    const tick = () => {
      const left = baseRef.current - (Date.now() - startedAtRef.current);
      if (left <= 0) {
        setRemaining(0);
        setStatus('COMPLETED');
      } else {
        setRemaining(left);
      }
    };
    tick();
    const id = setInterval(tick, 250);
    return () => clearInterval(id);
  }, [status]);

  const start = useCallback(() => {
    const duration = getDurationMs(mode);
    baseRef.current = duration;
    startedAtRef.current = Date.now();
    setRemaining(duration);
    setStatus('RUNNING');
  }, [mode]);

  const pause = useCallback(() => {
    if (status !== 'RUNNING') return;
    // recompute exactly so resume continues from the precise paused time
    setRemaining(Math.max(0, baseRef.current - (Date.now() - startedAtRef.current)));
    setStatus('PAUSED');
  }, [status]);

  const resume = useCallback(() => {
    if (status !== 'PAUSED') return;
    baseRef.current = remaining;
    startedAtRef.current = Date.now();
    setStatus('RUNNING');
  }, [status, remaining]);

  const reset = useCallback(() => {
    const duration = getDurationMs(mode);
    baseRef.current = duration;
    setRemaining(duration);
    setStatus('IDLE');
  }, [mode]);

  const switchMode = useCallback((m: TimerMode) => {
    const duration = getDurationMs(m);
    setMode(m);
    baseRef.current = duration;
    setRemaining(duration);
    setStatus('IDLE');
  }, []);

  return { mode, status, remaining, start, pause, resume, reset, switchMode, startedAt: startedAtRef.current };
}