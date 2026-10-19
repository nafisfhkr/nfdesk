import { motion } from 'framer-motion';
import { Play, Pause, Square, RotateCcw, Coffee, Brain } from 'lucide-react';
import type { useTimer } from '../../hooks/useTimer';
import { formatTime, getDurationMin, getDurationMs } from '../../hooks/useTimer';

type TimerController = ReturnType<typeof useTimer>;

interface FocusTimerViewProps {
  timer: TimerController;
  currentTime: string;
  taskTitle: string;
  onTaskTitleChange(value: string): void;
  logStatus: 'IDLE' | 'SAVING' | 'DONE' | 'ERROR';
  onSaveFocusLog(): void;
}

export default function FocusTimerView({
  timer,
  currentTime,
  taskTitle,
  onTaskTitleChange,
  logStatus,
  onSaveFocusLog,
}: FocusTimerViewProps) {
  const { mode, status, remaining, start, pause, resume, reset, switchMode } = timer;
  const isRunning = status === 'RUNNING';
  const isFocusCompleted = status === 'COMPLETED' && mode === 'FOCUS';

  return (
    <motion.div
      key="timer"
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 10 }}
      transition={{ duration: 0.15 }}
      className="flex-1 flex flex-col justify-between overflow-y-auto custom-scrollbar pr-1"
    >
      {/* Top Info Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-1.5 text-xs text-slate-400">
          <span className={`w-1.5 h-1.5 rounded-full ${isRunning ? 'bg-rose-400 animate-ping' : 'bg-emerald-400'}`} />
          <span>{isRunning ? 'Focusing' : status === 'PAUSED' ? 'Paused' : 'Ready'}</span>
        </div>
        <div className="flex items-center gap-1 text-[11px] font-mono text-slate-300 bg-white/5 px-2.5 py-1 rounded-lg border border-white/10">
          <span>{currentTime || '--:--:--'}</span>
        </div>
      </div>

      {/* Mode Toggle */}
      <div className="flex items-center justify-center mt-2">
        <div className="flex items-center gap-1 p-1 rounded-xl bg-white/5 border border-white/10">
          <button
            onClick={() => switchMode('FOCUS')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-200 ${
              mode === 'FOCUS'
                ? 'bg-gradient-to-r from-amber-600 to-amber-700 text-white shadow-md shadow-amber-600/25'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Brain className="w-3.5 h-3.5" />
            Focus
            <span className="opacity-70 text-[10px]">{getDurationMin('FOCUS')}m</span>
          </button>
          <button
            onClick={() => switchMode('BREAK')}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-200 ${
              mode === 'BREAK'
                ? 'bg-gradient-to-r from-emerald-500 to-teal-500 text-white shadow-md shadow-emerald-500/25'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            <Coffee className="w-3.5 h-3.5" />
            Break
            <span className="opacity-70 text-[10px]">{getDurationMin('BREAK')}m</span>
          </button>
        </div>
      </div>

      {/* Current Task Input */}
      <div className="relative mt-2">
        <input
          type="text"
          value={taskTitle}
          onChange={(e) => onTaskTitleChange(e.target.value)}
          placeholder={isRunning ? taskTitle || 'Focusing…' : 'What are you focusing on?'}
          readOnly={isRunning}
          disabled={isRunning}
          className={`w-full text-center text-sm py-2.5 bg-transparent outline-none border-b transition-all duration-300 placeholder:text-slate-500 ${
            isRunning
              ? 'text-slate-200 font-semibold border-amber-500/40 focus:border-amber-400'
              : 'text-slate-100 border-white/10 hover:border-white/20 focus:border-amber-400 focus:glow-amber'
          }`}
        />
      </div>

      {/* Timer Display */}
      <div className="my-auto py-4 flex flex-col items-center justify-center">
        <div className={`relative flex items-center justify-center ${isRunning ? 'animate-glow-pulse' : ''}`}>
          {/* Circular Progress Ring */}
          <svg className="absolute w-52 h-52 -rotate-90" viewBox="0 0 200 200">
            <defs>
              <linearGradient id="ringGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                <stop offset="0%" stopColor={mode === 'BREAK' ? '#34d399' : '#d97706'} />
                <stop offset="100%" stopColor={mode === 'BREAK' ? '#2dd4bf' : '#f59e0b'} />
              </linearGradient>
            </defs>
            <circle cx="100" cy="100" r="88" fill="none" stroke="rgba(255,255,255,0.06)" strokeWidth="6" />
            <circle
              cx="100"
              cy="100"
              r="88"
              fill="none"
              stroke="url(#ringGrad)"
              strokeWidth="6"
              strokeLinecap="round"
              strokeDasharray={2 * Math.PI * 88}
              strokeDashoffset={2 * Math.PI * 88 * (1 - remaining / getDurationMs(mode))}
              className="transition-[stroke-dashoffset] duration-300 ease-linear"
            />
          </svg>

          <div className="w-44 h-44 rounded-full flex flex-col items-center justify-center glass-card">
            <span
              className={`font-bold tabular-nums tracking-tight ${isRunning ? 'animate-pulse-subtle' : ''}`}
              style={{ fontSize: remaining >= 10 * 60 * 1000 ? '3.4rem' : '3.1rem' }}
            >
              {formatTime(remaining)}
            </span>
            <span className="text-[10px] uppercase tracking-[0.2em] text-slate-400 mt-1">
              {mode === 'BREAK' ? 'Break' : 'Focus'} session
            </span>
          </div>
        </div>

        {/* Controls */}
        <div className="flex items-center gap-3 mt-6">
          {status === 'IDLE' && (
            <button
              onClick={start}
              className="flex items-center gap-2 px-8 py-3 rounded-xl bg-gradient-to-r from-amber-600 to-amber-700 text-white text-sm font-bold shadow-lg shadow-amber-600/30 hover:brightness-110 active:scale-95 transition-all duration-200"
            >
              <Play className="w-4 h-4 fill-current" />
              Start Focus
            </button>
          )}

          {isRunning && (
            <button
              onClick={pause}
              className="flex items-center gap-2 px-8 py-3 rounded-xl bg-white/10 text-slate-100 text-sm font-bold border border-white/10 hover:bg-white/15 active:scale-95 transition-all duration-200"
            >
              <Pause className="w-4 h-4 fill-current" />
              Pause
            </button>
          )}

          {status === 'PAUSED' && (
            <>
              <button
                onClick={resume}
                className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-amber-600 to-amber-700 text-white text-sm font-bold shadow-lg shadow-amber-600/30 hover:brightness-110 active:scale-95 transition-all duration-200"
              >
                <Play className="w-4 h-4 fill-current" />
                Resume
              </button>
              <button
                onClick={reset}
                className="flex items-center gap-2 px-4 py-3 rounded-xl bg-white/10 text-slate-400 text-sm font-bold border border-white/10 hover:text-rose-400 hover:border-rose-500/30 active:scale-95 transition-all duration-200"
              >
                <Square className="w-3.5 h-3.5" />
              </button>
            </>
          )}

          {isFocusCompleted && (
            <button
              onClick={onSaveFocusLog}
              disabled={logStatus === 'SAVING'}
              className={`flex items-center gap-2 px-4 py-3 rounded-xl text-sm font-bold border transition-all duration-200 ${
                logStatus === 'DONE'
                  ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'
                  : logStatus === 'ERROR'
                    ? 'bg-rose-500/20 text-rose-400 border-rose-500/30'
                    : 'bg-white/10 text-slate-100 border-white/10 hover:bg-white/15 active:scale-95'
              }`}
            >
              {logStatus === 'DONE' ? '✓ Logged!' : logStatus === 'ERROR' ? 'Error!' : '💾 Save Log to Obsidian'}
            </button>
          )}

          {status === 'COMPLETED' && (
            <button
              onClick={reset}
              className="flex items-center gap-2 px-8 py-3 rounded-xl bg-gradient-to-r from-amber-600 to-amber-700 text-white text-sm font-bold shadow-lg shadow-amber-600/30 hover:brightness-110 active:scale-95 transition-all duration-200"
            >
              <RotateCcw className="w-4 h-4" />
              Restart
            </button>
          )}
        </div>
      </div>
    </motion.div>
  );
}
