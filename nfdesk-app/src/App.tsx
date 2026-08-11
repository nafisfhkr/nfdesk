import { useState, useEffect } from 'react';
import { X, Minus, Sparkles, Pin, LayoutGrid, Clock, ShieldCheck } from 'lucide-react';

function App() {
  const [time, setTime] = useState<string>('');
  const [isAlwaysOnTop, setIsAlwaysOnTop] = useState(true);

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
            <div className="w-6 h-6 rounded-lg bg-gradient-to-tr from-indigo-600 to-violet-500 flex items-center justify-center shadow-md shadow-indigo-500/20">
              <Sparkles className="w-3.5 h-3.5 text-white" />
            </div>
            <span className="text-xs font-bold tracking-wide text-slate-100 drop-shadow-sm">
              NFDesk
            </span>
            <span className="text-[10px] px-1.5 py-0.5 rounded-full bg-indigo-500/20 text-indigo-300 font-semibold border border-indigo-500/30">
              MVP
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
        <div className="flex-1 p-4 flex flex-col justify-between z-10 overflow-y-auto">
          
          {/* Top Info Header */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 text-xs text-slate-400">
              <ShieldCheck className="w-4 h-4 text-emerald-400" />
              <span>Foundation Active</span>
            </div>
            <div className="flex items-center gap-1 text-[11px] font-mono text-slate-300 bg-white/5 px-2.5 py-1 rounded-lg border border-white/10">
              <Clock className="w-3 h-3 text-indigo-400" />
              <span>{time || '--:--:--'}</span>
            </div>
          </div>

          {/* Center Card Display */}
          <div className="my-auto py-6 px-4 rounded-xl glass-card text-center flex flex-col items-center justify-center space-y-3">
            <div className="p-3 rounded-full bg-gradient-to-br from-indigo-500/20 to-purple-500/20 border border-indigo-500/30 text-indigo-400 shadow-inner">
              <LayoutGrid className="w-6 h-6 animate-pulse-subtle" />
            </div>

            <div>
              <h2 className="text-sm font-bold text-slate-100 tracking-wide">
                Frameless Window Foundation
              </h2>
              <p className="text-xs text-slate-400 mt-1 max-w-[240px] leading-relaxed">
                Tauri v2 core window initialized with drag region, transparency, and top-layer pin state.
              </p>
            </div>

            <div className="pt-1 flex items-center justify-center gap-2">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-ping" />
              <span className="text-[11px] font-medium text-emerald-400">Ready for Pomodoro Phase</span>
            </div>
          </div>

          {/* Footer Bar */}
          <div className="pt-2 border-t border-white/5 flex items-center justify-between text-[10px] text-slate-400">
            <span>NFDesk Floating Widget</span>
            <span className="font-mono text-slate-400">350 x 500 px</span>
          </div>

        </div>

      </div>
    </div>
  );
}

export default App;
