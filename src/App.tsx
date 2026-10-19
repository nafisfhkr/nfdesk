import { useState, useEffect } from 'react';
import { X, Minus, Pin, Clock, CalendarDays, Edit3, Settings, Maximize2, Minimize2 } from 'lucide-react';
import { AnimatePresence, motion } from 'framer-motion';
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart';
import { useTimer, getDurationMin, DURATION_KEYS } from './hooks/useTimer';
import { appendDailyNote, timeStamp } from './lib/markdown';
import PlannerView, { type PlannerFocusTask } from './features/planner/PlannerView';
import FocusTimerView from './features/timer/FocusTimerView';
import NoteView from './components/NoteView';
import VaultSetup from './components/VaultSetup';
import { notifyTimerCompletion } from './lib/notifications';

type ActiveView = 'PLANNER' | 'TIMER' | 'NOTE';

function App() {
  const [time, setTime] = useState<string>('');
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(true);
  const [isMaximized, setIsMaximized] = useState(false);
  const [task, setTask] = useState('');
  const [activeView, setActiveView] = useState<ActiveView>('PLANNER');
  const [focusTask, setFocusTask] = useState<PlannerFocusTask | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [logStatus, setLogStatus] = useState<'IDLE' | 'SAVING' | 'DONE' | 'ERROR'>('IDLE');
  const [autoStartEnabled, setAutoStartEnabled] = useState(false);
  const [focusMinutes, setFocusMinutes] = useState(() => getDurationMin('FOCUS'));
  const [breakMinutes, setBreakMinutes] = useState(() => getDurationMin('BREAK'));

  const timer = useTimer();
  const { mode, status, start, pause, resume } = timer;

  useEffect(() => {
    isEnabled().then(setAutoStartEnabled).catch(() => setAutoStartEnabled(false));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const initWindow = async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        setIsMaximized(await win.isMaximized());
        unlisten = await win.onResized(async () => {
          try {
            setIsMaximized(await win.isMaximized());
          } catch {}
        });
      } catch {}
    };
    initWindow();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  // Global keyboard shortcuts: Esc closes Settings, Space toggles timer, 1/2/3 switches views.
  // Must not interfere with typing in inputs.
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (showSettings) {
          e.preventDefault();
          setShowSettings(false);
        }
        return;
      }

      const target = e.target as HTMLElement | null;
      const isTyping =
        target?.tagName === 'INPUT' ||
        target?.tagName === 'TEXTAREA' ||
        target?.tagName === 'SELECT' ||
        target?.isContentEditable;

      if (isTyping) return;

      if (!showSettings) {
        if (e.key === '1') {
          e.preventDefault();
          setActiveView('PLANNER');
          return;
        }
        if (e.key === '2') {
          e.preventDefault();
          setActiveView('TIMER');
          return;
        }
        if (e.key === '3') {
          e.preventDefault();
          setActiveView('NOTE');
          return;
        }
      }

      if (e.key === ' ' && activeView === 'TIMER') {
        e.preventDefault();
        if (status === 'IDLE') start();
        else if (status === 'RUNNING') pause();
        else if (status === 'PAUSED') resume();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showSettings, activeView, status, start, pause, resume]);

  const toggleAutoStart = async () => {
    if (autoStartEnabled) {
      await disable();
      setAutoStartEnabled(false);
    } else {
      await enable();
      setAutoStartEnabled(true);
    }
  };

  const saveSettings = () => {
    localStorage.setItem(DURATION_KEYS.FOCUS, focusMinutes.toString());
    localStorage.setItem(DURATION_KEYS.BREAK, breakMinutes.toString());
    timer.reset();
    setShowSettings(false);
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

  const handleToggleMaximize = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      const isMax = await win.isMaximized();
      if (isMax) {
        await win.unmaximize();
        setIsMaximized(false);
      } else {
        await win.maximize();
        setIsMaximized(true);
      }
    } catch (err) {
      console.error('Failed to toggle maximize:', err);
    }
  };

  const saveFocusLog = async () => {
    setLogStatus('SAVING');
    try {
      const end = new Date();
      const startDate = timer.startedAt ? new Date(timer.startedAt) : end;
      await appendDailyNote(`- **${timeStamp(startDate)}–${timeStamp(end)}** — ${task || 'Focus session'}`);
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

  // Trigger audio chime & Windows notification when timer reaches 00:00
  useEffect(() => {
    if (status === 'COMPLETED') {
      notifyTimerCompletion(mode, task);
    }
  }, [status, mode, task]);

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

  const handlePlannerStartFocus = (selected: PlannerFocusTask) => {
    setFocusTask(selected);
    setTask(selected.title);
    setActiveView('TIMER');
    if (status === 'IDLE') start();
  };

  void focusTask;

  return (
    <div className={`flex flex-col w-screen h-screen bg-transparent ${isMaximized ? 'p-1' : 'p-3'} select-none overflow-hidden font-sans transition-all duration-200`}>
      {/* Outer Glow & Glassmorphism Container */}
      <div className={`relative flex-1 ${isMaximized ? 'rounded-xl' : 'rounded-2xl'} glass-panel flex flex-col overflow-hidden border ${isMaximized ? 'border-stone-800/40' : 'border-stone-800/80'} shadow-2xl transition-all duration-300`}>
        {/* Subtle Ambient Glows - Warm paper / desk lighting */}
        <div className="absolute -top-16 -left-16 w-48 h-48 bg-amber-600/10 rounded-full blur-3xl pointer-events-none" />
        <div className="absolute -bottom-16 -right-16 w-48 h-48 bg-stone-500/10 rounded-full blur-3xl pointer-events-none" />

        {/* Drag Region Header */}
        <div
          data-tauri-drag-region
          onDoubleClick={handleToggleMaximize}
          className="h-11 glass-header flex items-center justify-between px-3 cursor-grab active:cursor-grabbing border-b border-stone-800/80 z-20"
        >
          {/* Brand & App Name */}
          <div className="flex items-center gap-2 pointer-events-none">
            <span className="text-xs font-bold tracking-wide text-stone-200 drop-shadow-sm">NFDesk</span>
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-stone-800 text-stone-300 font-semibold border border-stone-700">
              FOCUS
            </span>
          </div>

          {/* Control Buttons */}
          <div className="flex items-center gap-1.5">
            <button
              onClick={toggleAlwaysOnTop}
              title={isAlwaysOnTop ? 'Always on top (Active)' : 'Pin on top'}
              className={`p-1 rounded-md transition-all duration-200 ${
                isAlwaysOnTop
                  ? 'text-amber-300 bg-amber-500/15 border border-amber-500/30'
                  : 'text-stone-400 hover:text-stone-200 hover:bg-stone-800/60'
              }`}
            >
              <Pin className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={() => setShowSettings(true)}
              title="Settings"
              className="p-1 rounded-md text-stone-400 hover:text-stone-200 hover:bg-stone-800/60 transition-all duration-200"
            >
              <Settings className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={handleMinimize}
              title="Minimize"
              className="p-1 rounded-md text-stone-400 hover:text-stone-200 hover:bg-stone-800/60 transition-all duration-200"
            >
              <Minus className="w-3.5 h-3.5" />
            </button>

            <button
              onClick={handleToggleMaximize}
              title={isMaximized ? 'Restore size' : 'Maximize'}
              className="p-1 rounded-md text-stone-400 hover:text-stone-200 hover:bg-stone-800/60 transition-all duration-200"
            >
              {isMaximized ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
            </button>

            <button
              onClick={handleClose}
              title="Close"
              className="p-1 rounded-md text-stone-400 hover:text-rose-400 hover:bg-rose-500/20 transition-all duration-200"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* App Content Body */}
        <div className="flex-1 p-4 flex flex-col justify-between z-10 overflow-hidden relative">
          <AnimatePresence mode="wait">
            {activeView === 'PLANNER' && (
              <motion.div
                key="planner"
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 10 }}
                transition={{ duration: 0.15 }}
                className="flex-1 w-full h-full"
              >
                <PlannerView onStartFocus={handlePlannerStartFocus} />
              </motion.div>
            )}

            {activeView === 'TIMER' && (
              <FocusTimerView
                timer={timer}
                currentTime={time}
                taskTitle={task}
                onTaskTitleChange={setTask}
                logStatus={logStatus}
                onSaveFocusLog={saveFocusLog}
              />
            )}

            {activeView === 'NOTE' && (
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
          <div className="flex bg-stone-900/80 p-1 rounded-xl border border-stone-800/80 mb-2 mt-2">
            <button
              onClick={() => setActiveView('PLANNER')}
              className={`group flex-1 py-1.5 px-3 rounded-lg text-xs font-medium transition-all flex items-center justify-center gap-1.5 ${
                activeView === 'PLANNER'
                  ? 'bg-stone-800 text-stone-100 border border-stone-700 shadow-sm'
                  : 'text-stone-400 hover:text-stone-200 hover:bg-stone-800/40'
              }`}
            >
              <CalendarDays className="w-3.5 h-3.5 text-stone-300" />
              <span>Planner</span>
              <kbd className="hidden group-hover:inline text-[9px] font-mono px-1 py-0.5 bg-stone-700/60 rounded text-stone-400">
                1
              </kbd>
            </button>

            <button
              onClick={() => setActiveView('TIMER')}
              className={`group flex-1 py-1.5 px-3 rounded-lg text-xs font-medium transition-all flex items-center justify-center gap-1.5 ${
                activeView === 'TIMER'
                  ? 'bg-stone-800 text-stone-100 border border-stone-700 shadow-sm'
                  : 'text-stone-400 hover:text-stone-200 hover:bg-stone-800/40'
              }`}
            >
              <Clock className="w-3.5 h-3.5 text-stone-300" />
              <span>Timer</span>
              <kbd className="hidden group-hover:inline text-[9px] font-mono px-1 py-0.5 bg-stone-700/60 rounded text-stone-400">
                2
              </kbd>
            </button>

            <button
              onClick={() => setActiveView('NOTE')}
              className={`group flex-1 py-1.5 px-3 rounded-lg text-xs font-medium transition-all flex items-center justify-center gap-1.5 ${
                activeView === 'NOTE'
                  ? 'bg-stone-800 text-stone-100 border border-stone-700 shadow-sm'
                  : 'text-stone-400 hover:text-stone-200 hover:bg-stone-800/40'
              }`}
            >
              <Edit3 className="w-3.5 h-3.5 text-stone-300" />
              <span>Note</span>
              <kbd className="hidden group-hover:inline text-[9px] font-mono px-1 py-0.5 bg-stone-700/60 rounded text-stone-400">
                3
              </kbd>
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
            className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
            onClick={() => setShowSettings(false)}
          >
            <motion.div
              initial={{ opacity: 0, y: 30, scale: 0.95 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: 20, scale: 0.95 }}
              transition={{ type: 'spring', stiffness: 300, damping: 25 }}
              onClick={(e) => e.stopPropagation()}
              className="glass-panel w-full max-w-sm rounded-2xl border border-stone-800 p-5 shadow-2xl"
            >
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-sm font-bold text-stone-100 flex items-center gap-2">
                  <Settings className="w-4 h-4 text-amber-400" />
                  Settings
                </h2>
                <button
                  onClick={() => setShowSettings(false)}
                  className="p-1 rounded-md text-stone-400 hover:text-stone-200 hover:bg-stone-800/60 transition-all"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              <div className="mb-4">
                <VaultSetup />
              </div>

              <div className="grid grid-cols-2 gap-3 mt-4">
                <div className="min-w-0">
                  <label className="block text-[11px] uppercase tracking-wider text-stone-400 mb-1.5 truncate">
                    Focus (min)
                  </label>
                  <input
                    type="number"
                    min={1}
                    value={focusMinutes}
                    onChange={(e) => setFocusMinutes(Number(e.target.value))}
                    placeholder="25"
                    className="w-full text-sm py-2.5 px-3 rounded-xl bg-white/5 border border-stone-700/80 text-stone-100 outline-none placeholder:text-stone-500 focus:border-amber-400/80 focus:ring-1 focus:ring-amber-400/20 transition-all"
                  />
                </div>
                <div className="min-w-0">
                  <label className="block text-[11px] uppercase tracking-wider text-stone-400 mb-1.5 truncate">
                    Break (min)
                  </label>
                  <input
                    type="number"
                    min={1}
                    value={breakMinutes}
                    onChange={(e) => setBreakMinutes(Number(e.target.value))}
                    placeholder="5"
                    className="w-full text-sm py-2.5 px-3 rounded-xl bg-white/5 border border-stone-700/80 text-stone-100 outline-none placeholder:text-stone-500 focus:border-amber-400/80 focus:ring-1 focus:ring-amber-400/20 transition-all"
                  />
                </div>
              </div>

              <button
                onClick={toggleAutoStart}
                className="flex items-center justify-between w-full mt-4 py-2.5 px-3 rounded-xl bg-white/5 border border-stone-700/80 hover:bg-white/10 transition-all"
              >
                <span className="flex items-center gap-2 text-sm text-stone-200">
                  <span className={`w-2 h-2 rounded-full ${autoStartEnabled ? 'bg-emerald-400' : 'bg-stone-500'}`} />
                  Launch at startup
                </span>
                <span
                  className={`relative w-9 h-5 rounded-full transition-colors duration-200 ${
                    autoStartEnabled ? 'bg-amber-600' : 'bg-white/15'
                  }`}
                >
                  <span
                    className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-all duration-200 ${
                      autoStartEnabled ? 'left-[18px]' : 'left-0.5'
                    }`}
                  />
                </span>
              </button>

              <div className="flex justify-between items-center gap-2 mt-4">
                <span className="text-[11px] text-stone-500 font-mono">v0.1.4</span>
                <div className="flex gap-2">
                  <button
                    onClick={() => setShowSettings(false)}
                    className="px-4 py-2 rounded-xl text-xs font-bold text-stone-400 border border-stone-700/80 hover:bg-stone-800/60 transition-all"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={saveSettings}
                    className="px-4 py-2 rounded-xl text-xs font-bold bg-stone-800 hover:bg-stone-700 text-stone-100 border border-stone-700 hover:border-stone-600 shadow-md active:scale-95 transition-all"
                  >
                    Save
                  </button>
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
