import { useState, useEffect } from 'react';
import { X, Minus, Pin, Play, Pause, Square, RotateCcw, Coffee, Brain, Clock, CheckSquare, Edit3, Settings } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTimer, formatTime } from './hooks/useTimer';
import { getVaultPath, setVaultPath, appendToMarkdown, timeStamp } from './lib/markdown';
import TasksView from './components/TasksView';
import NoteView from './components/NoteView';
import Mascot, { MascotStatus } from './components/Mascot';

function App() {
  const [time, setTime] = useState<string>('');
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(true);
  const [task, setTask] = useState('');
  const [activeTab, setActiveTab] = useState<'TIMER' | 'TASKS' | 'NOTE'>('TIMER');
  const [showSettings, setShowSettings] = useState(false);
  const [vaultPath, setVaultPathState] = useState(getVaultPath());
  const [logStatus, setLogStatus] = useState<'IDLE' | 'SAVING' | 'DONE' | 'ERROR'>('IDLE');

  const { mode, status, remaining, start, pause, resume, reset, switchMode, startedAt } = useTimer();
  const isRunning = status === 'RUNNING';
  const isFocusCompleted = status === 'COMPLETED' && mode === 'FOCUS';

  const getMascotStatus = (): MascotStatus => {
    if (status === 'COMPLETED') return 'SUCCESS';
    if (mode === 'BREAK') return 'BREAK';
    if (status === 'RUNNING') return 'FOCUS';
    return 'IDLE';
  };

  useEffect(() => {
    const updateTime = () => {
      const now = new Date();
      setTime(now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' }));
    };
    updateTime();
    const interval = setInterval(updateTime, 1000);
    return () => clearInterval(interval);
  }, []);

  const handleClose = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } catch {
      console.log('Window close called (Browser mode fallback)');
    }
  };

  const handleMinimize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch {
      console.log('Window minimize called (Browser mode fallback)');
    }
  };

  const saveFocusLog = async () => {
    setLogStatus('SAVING');
    try {
      const end = new Date();
      const start = startedAt ? new Date(startedAt) : end;
      await appendToMarkdown(`- **${timeStamp(start)}–${timeStamp(end)}** — ${task || 'Focus session'}`);
      setLogStatus('DONE');
    } catch (e) {
      console.error(e);
      setLogStatus('ERROR');
    }
  };

  // Reset the log button state whenever the session is restarted or mode switches
  useEffect(() => {
    setLogStatus('IDLE');
  }, [status, mode]);

  const toggleAlwaysOnTop = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      const nextState = !isAlwaysOnTop;
      await win.setAlwaysOnTop(nextState);
      setIsAlwaysOnTop(nextState);
    } catch {
      setIsAlwaysOnTop(!isAlwaysOnTop);
    }
  };

  return (
    <div className="flex flex-col w-screen h-screen bg-transparent p-3 select-none overflow-hidden font-sans">
      {/* Outer Glow & Glassmorphism Container */}
      <div className="relative flex-1 rounded-2xl glass-panel flex flex-col overflow-hidden border border-slate-700/50 shadow-2xl transition-all duration-300">

        {/* Subtle Ambient Top Glow */}
        <div className="absolute -top-16 -left-16 w-48 h-48 bg-indigo-500/20 rounded-full blur-3xl pointer-events-none" />
        <div className="absolute -bottom-16 -right-16 w-48 h-48 bg-purple-500/20 rounded-full blur-3xl pointer-events-none" />

        {/* Drag Region Header */}
        <div
          data-tauri-drag-region
          className="h-11 glass-header flex items-center justify-between px-3 cursor-grab active:cursor-grabbing border-b border-white/10 z-20"
        >
          {/* Brand & App Name */}
          <div className="flex items-center gap-2 pointer-events-none">
            <span className="text-xs font-bold tracking-wide text-slate-100 drop-shadow-sm">
              NFDesk
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 font-semibold border border-indigo-500/30">
              FOCUS
            </span>
          </div>

          {/* Control Buttons */}
          <div className="flex items-center gap-1.5">
            <button
              onClick={toggleAlwaysOnTop}
              title={isAlwaysOnTop ? "Always on top (Active)" : "Pin on top"}
              className={`p-1 rounded-md transition-all duration-200 ${
                isAlwaysOnTop
                  ? 'text-indigo-400 bg-indigo-500/20 border border-indigo-500/30'
                  : 'text-slate-400 hover:text-slate-200 hover:bg-white/10'
              }`}
            >
              <Pin className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={() => setShowSettings(true)}
              title="Settings"
              className="p-1 rounded-md text-slate-400 hover:text-slate-200 hover:bg-white/10 transition-all duration-200"
            >
              <Settings className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={handleMinimize}
              title="Minimize"
              className="p-1 rounded-md text-slate-400 hover:text-slate-200 hover:bg-white/10 transition-all duration-200"
            >
              <Minus className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={handleClose}
              title="Close"
              className="p-1 rounded-md text-slate-400 hover:text-rose-400 hover:bg-rose-500/20 transition-all duration-200"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* App Content Body */}
        <div className="flex-1 p-4 flex flex-col justify-between z-10 overflow-hidden relative">
          
          <AnimatePresence mode="wait">
            {activeTab === 'TIMER' && (
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
                    <span>{time || '--:--:--'}</span>
                  </div>
                </div>

                {/* Mode Toggle */}
                <div className="flex items-center justify-center mt-2">
                  <div className="flex items-center gap-1 p-1 rounded-xl bg-white/5 border border-white/10">
                    <button
                      onClick={() => switchMode('FOCUS')}
                      className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all duration-200 ${
                        mode === 'FOCUS'
                          ? 'bg-gradient-to-r from-indigo-600 to-violet-500 text-white shadow-md shadow-indigo-500/25'
                          : 'text-slate-400 hover:text-slate-200'
                      }`}
                    >
                      <Brain className="w-3.5 h-3.5" />
                      Focus
                      <span className="opacity-70 text-[10px]">25m</span>
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
                      <span className="opacity-70 text-[10px]">5m</span>
                    </button>
                  </div>
                </div>

                {/* Current Task Input */}
                <div className="relative mt-2">
                  <input
                    type="text"
                    value={task}
                    onChange={(e) => setTask(e.target.value)}
                    placeholder={isRunning ? task || 'Focusing…' : "What are you focusing on?"}
                    readOnly={isRunning}
                    disabled={isRunning}
                    className={`w-full text-center text-sm py-2.5 bg-transparent outline-none border-b transition-all duration-300 placeholder:text-slate-500 ${
                      isRunning
                        ? 'text-slate-200 font-semibold border-indigo-500/40 focus:border-indigo-400'
                        : 'text-slate-100 border-white/10 hover:border-white/20 focus:border-indigo-400 focus:glow-indigo'
                    }`}
                  />
                </div>

                {/* Timer Display */}
                <div className="my-auto py-4 flex flex-col items-center justify-center">
                  <div className={`relative flex items-center justify-center ${
                    isRunning ? 'animate-glow-pulse' : ''
                  }`}>
                    {/* Circular Progress Ring */}
                    <svg className="absolute w-52 h-52 -rotate-90" viewBox="0 0 200 200">
                      <defs>
                        <linearGradient id="ringGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                          <stop offset="0%" stopColor={mode === 'BREAK' ? '#34d399' : '#6366f1'} />
                          <stop offset="100%" stopColor={mode === 'BREAK' ? '#2dd4bf' : '#8b5cf6'} />
                        </linearGradient>
                      </defs>
                      <circle cx="100" cy="100" r="88" fill="none" stroke="rgba(255,255,255,0.06)" strokeWidth="6" />
                      <circle
                        cx="100" cy="100" r="88" fill="none"
                        stroke="url(#ringGrad)" strokeWidth="6" strokeLinecap="round"
                        strokeDasharray={2 * Math.PI * 88}
                        strokeDashoffset={2 * Math.PI * 88 * (1 - remaining / (mode === 'BREAK' ? 5 * 60 * 1000 : 25 * 60 * 1000))}
                        className="transition-[stroke-dashoffset] duration-300 ease-linear"
                      />
                    </svg>

                    <div className="w-44 h-44 rounded-full flex flex-col items-center justify-center glass-card">
                      <span className={`font-bold tabular-nums tracking-tight ${isRunning ? 'animate-pulse-subtle' : ''}`}
                            style={{ fontSize: remaining >= 10 * 60 * 1000 ? '3.4rem' : '3.1rem' }}>
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
                        className="flex items-center gap-2 px-8 py-3 rounded-xl bg-gradient-to-r from-indigo-600 to-violet-500 text-white text-sm font-bold shadow-lg shadow-indigo-500/30 hover:brightness-110 active:scale-95 transition-all duration-200"
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
                          className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-indigo-600 to-violet-500 text-white text-sm font-bold shadow-lg shadow-indigo-500/30 hover:brightness-110 active:scale-95 transition-all duration-200"
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
                        onClick={saveFocusLog}
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
                        className="flex items-center gap-2 px-8 py-3 rounded-xl bg-gradient-to-r from-indigo-600 to-violet-500 text-white text-sm font-bold shadow-lg shadow-indigo-500/30 hover:brightness-110 active:scale-95 transition-all duration-200"
                      >
                        <RotateCcw className="w-4 h-4" />
                        Restart
                      </button>
                    )}
                  </div>
                </div>
              </motion.div>
            )}

            {activeTab === 'TASKS' && (
              <motion.div
                key="tasks"
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 10 }}
                transition={{ duration: 0.15 }}
                className="flex-1 w-full h-full"
              >
                <TasksView />
              </motion.div>
            )}

            {activeTab === 'NOTE' && (
              <motion.div
                key="note"
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 10 }}
                transition={{ duration: 0.15 }}
                className="flex-1 w-full h-full"
              >
                <NoteView />
              </motion.div>
            )}
          </AnimatePresence>

          {/* Bottom Navigation Bar */}
          <div className="mt-2 pt-2 border-t border-white/5 flex items-center justify-around">
            <button
              onClick={() => setActiveTab('TIMER')}
              className={`p-2.5 flex-1 rounded-xl flex flex-col items-center justify-center gap-1 transition-all duration-300 ${
                activeTab === 'TIMER'
                  ? 'text-indigo-300 bg-indigo-500/10 shadow-inner'
                  : 'text-slate-500 hover:text-slate-300 hover:bg-white/5'
              }`}
            >
              <Clock className="w-5 h-5" />
            </button>
            <button
              onClick={() => setActiveTab('TASKS')}
              className={`p-2.5 flex-1 rounded-xl flex flex-col items-center justify-center gap-1 transition-all duration-300 ${
                activeTab === 'TASKS'
                  ? 'text-indigo-300 bg-indigo-500/10 shadow-inner'
                  : 'text-slate-500 hover:text-slate-300 hover:bg-white/5'
              }`}
            >
              <CheckSquare className="w-5 h-5" />
            </button>
            <button
              onClick={() => setActiveTab('NOTE')}
              className={`p-2.5 flex-1 rounded-xl flex flex-col items-center justify-center gap-1 transition-all duration-300 ${
                activeTab === 'NOTE'
                  ? 'text-indigo-300 bg-indigo-500/10 shadow-inner'
                  : 'text-slate-500 hover:text-slate-300 hover:bg-white/5'
              }`}
            >
              <Edit3 className="w-5 h-5" />
            </button>
          </div>

        </div>
      </div>

      {/* Settings Modal */}
      <AnimatePresence>
        {showSettings && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 bg-black/40 backdrop-blur-sm flex items-center justify-center p-4"
            onClick={() => setShowSettings(false)}
          >
            <motion.div
              initial={{ opacity: 0, y: 30, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 20, scale: 0.95 }}
              transition={{ type: 'spring', stiffness: 300, damping: 25 }}
              onClick={(e) => e.stopPropagation()}
              className="glass-panel w-full max-w-sm rounded-2xl border border-white/10 p-5 shadow-2xl"
            >
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                  <Settings className="w-4 h-4 text-indigo-400" />
                  Settings
                </h2>
                <button
                  onClick={() => setShowSettings(false)}
                  className="p-1 rounded-md text-slate-400 hover:text-slate-200 hover:bg-white/10 transition-all"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              <label className="block text-[11px] uppercase tracking-wider text-slate-400 mb-1.5">
                Obsidian Vault Path
              </label>
              <input
                type="text"
                value={vaultPath}
                onChange={(e) => setVaultPathState(e.target.value)}
                placeholder="C:\Users\Nafis\Documents\Obsidian"
                className="w-full text-sm py-2.5 px-3 rounded-xl bg-white/5 border border-white/10 text-slate-100 outline-none placeholder:text-slate-500 focus:border-indigo-400 focus:glow-indigo transition-all"
              />

              <div className="flex justify-end gap-2 mt-4">
                <button
                  onClick={() => setShowSettings(false)}
                  className="px-4 py-2 rounded-xl text-xs font-bold text-slate-400 border border-white/10 hover:bg-white/5 transition-all"
                >
                  Cancel
                </button>
                <button
                  onClick={() => {
                    setVaultPath(vaultPath);
                    setShowSettings(false);
                  }}
                  className="px-4 py-2 rounded-xl text-xs font-bold bg-gradient-to-r from-indigo-600 to-violet-500 text-white shadow-md shadow-indigo-500/25 hover:brightness-110 active:scale-95 transition-all"
                >
                  Save
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
