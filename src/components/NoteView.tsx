import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Save, Check } from 'lucide-react';

export default function NoteView() {
  const [note, setNote] = useState('');
  const [showToast, setShowToast] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    // Focus the textarea when the view mounts
    if (textareaRef.current) {
      textareaRef.current.focus();
    }
  }, []);

  const saveNote = () => {
    if (note.trim() === '') return;
    
    // TODO: Phase 2 - connect to Rust backend file-writing logic
    console.log("Saving note:", note);

    // Show toast and clear
    setNote('');
    setShowToast(true);
    
    // Hide toast after 2 seconds
    setTimeout(() => {
      setShowToast(false);
    }, 2000);
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
      <div className="absolute top-0 left-1/2 -translate-x-1/2 pointer-events-none">
        <AnimatePresence>
          {showToast && (
            <motion.div
              initial={{ opacity: 0, y: -20, scale: 0.9 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -10, scale: 0.9 }}
              className="bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 px-3 py-1.5 rounded-full text-xs flex items-center gap-1.5 backdrop-blur-md shadow-lg"
            >
              <Check className="w-3.5 h-3.5" />
              Note saved!
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
