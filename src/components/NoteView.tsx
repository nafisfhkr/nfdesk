import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Save, Check, AlertTriangle } from 'lucide-react';
import { appendDailyNote, formatErrorMessage, timeStamp } from '../lib/markdown';

type ToastState = { kind: 'success' | 'error'; msg: string } | null;

export default function NoteView() {
  const [note, setNote] = useState('');
  const [toast, setToast] = useState<ToastState>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    // Focus the textarea when the view mounts
    if (textareaRef.current) {
      textareaRef.current.focus();
    }
  }, []);

  useEffect(() => {
    if (!toast) return;
    const id = setTimeout(() => setToast(null), 2000);
    return () => clearTimeout(id);
  }, [toast]);

  const saveNote = async () => {
    if (note.trim() === '') return;
    try {
      await appendDailyNote(`- **${timeStamp()}** — ${note.trim()}`);
      setNote('');
      setToast({ kind: 'success', msg: 'Note saved!' });
    } catch (e) {
      setToast({ kind: 'error', msg: formatErrorMessage(e) });
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.ctrlKey) {
      saveNote();
    }
  };

  return (
    <div className="flex flex-col h-full relative">
      <textarea
        ref={textareaRef}
        value={note}
        onChange={(e) => setNote(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Type a quick note... (Ctrl+Enter to save)"
        className="flex-1 w-full bg-transparent resize-none outline-none text-sm text-slate-200 placeholder:text-slate-500 custom-scrollbar p-1"
        style={{ scrollbarWidth: 'thin' }}
      />

      {/* Toast Notification */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 pointer-events-none z-10">
        <AnimatePresence>
          {toast && (
            <motion.div
              key={toast.msg}
              initial={{ opacity: 0, y: -20, scale: 0.9 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -10, scale: 0.9 }}
              className={`px-3 py-1.5 rounded-full text-xs flex items-center gap-1.5 backdrop-blur-md shadow-lg ${
                toast.kind === 'success'
                  ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                  : 'bg-rose-500/20 text-rose-400 border border-rose-500/30'
              }`}
            >
              {toast.kind === 'success' ? (
                <Check className="w-3.5 h-3.5" />
              ) : (
                <AlertTriangle className="w-3.5 h-3.5" />
              )}
              {toast.msg}
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {/* Save Button */}
      <div className="flex justify-end mt-2">
        <button
          onClick={saveNote}
          disabled={note.trim() === ''}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all duration-300 ${
            note.trim() === ''
              ? 'bg-white/5 text-slate-500 cursor-not-allowed border border-white/5'
              : 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 hover:bg-indigo-500 hover:text-white hover:border-indigo-500 hover:shadow-lg hover:shadow-indigo-500/20 active:scale-95'
          }`}
        >
          <Save className="w-3.5 h-3.5" />
          Save Note
        </button>
      </div>
    </div>
  );
}
