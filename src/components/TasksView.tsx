import { useState, useEffect, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Check, Trash2, Plus, FolderOpen, Play } from 'lucide-react';
import { readTasksFromVault, saveTasksToVault, getVaultPath, getTasksFolder, todayFilename, type MarkdownTask } from '../lib/markdown';

export interface TasksViewProps {
  onFocusTask?: (taskTitle: string) => void;
}

function parseTaskTitle(rawTitle: string): { tag: string | null; title: string } {
  const match = rawTitle.match(/^\[(.*?)\]\s*(.*)$/);
  if (match) {
    return {
      tag: match[1],
      title: match[2] || rawTitle,
    };
  }
  return { tag: null, title: rawTitle };
}

export default function TasksView({ onFocusTask }: TasksViewProps) {
  const [tasks, setTasks] = useState<MarkdownTask[]>([]);
  const [newTaskTitle, setNewTaskTitle] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadTasks = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      setTasks(await readTasksFromVault());
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load tasks');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  useEffect(() => {
    const onFocus = () => loadTasks();
    window.addEventListener('focus', onFocus);
    return () => window.removeEventListener('focus', onFocus);
  }, [loadTasks]);

  const sync = async (next: MarkdownTask[]) => {
    setTasks(next);
    try {
      await saveTasksToVault(next);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save tasks');
    }
  };

  const addTask = () => {
    if (newTaskTitle.trim() === '') return;
    const newTask: MarkdownTask = {
      id: crypto.randomUUID(),
      title: newTaskTitle.trim(),
      completed: false,
    };
    sync([...tasks, newTask]);
    setNewTaskTitle('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      addTask();
    }
  };

  const toggleTask = (id: string) => {
    sync(tasks.map(t => t.id === id ? { ...t, completed: !t.completed } : t));
  };

  const deleteTask = (id: string) => {
    sync(tasks.filter(t => t.id !== id));
  };

  const vaultPath = getVaultPath();

  if (!vaultPath) {
    return (
      <div className="flex flex-col h-full items-center justify-center text-center text-slate-400 px-6">
        <FolderOpen className="w-10 h-10 mb-3 text-indigo-400/60" />
        <p className="text-sm">
          Please set your Obsidian Vault path in Settings to enable task synchronization.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden text-slate-200">
      <div className="flex-1 overflow-y-auto pr-1 pb-4 custom-scrollbar">
        {/* Date Header */}
        <div className="mb-3 flex items-center gap-2">
          <span className="text-[11px] font-semibold tracking-wide uppercase text-indigo-300/80 bg-indigo-500/10 border border-indigo-500/20 px-2.5 py-1 rounded-lg">
            Tasks • {todayFilename().replace('.md', '')}
          </span>
          <span className="text-[10px] text-slate-500">in {getTasksFolder()}</span>
        </div>

        {error && (
          <div className="mb-3 text-xs text-rose-400 bg-rose-500/10 border border-rose-500/20 px-3 py-2 rounded-xl">
            {error}
          </div>
        )}

        {/* Header / Input */}
        <div className="relative mb-4">
          <input
            type="text"
            value={newTaskTitle}
            onChange={(e) => setNewTaskTitle(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add new task..."
            className="w-full bg-white/5 border border-white/10 rounded-xl py-3 pl-4 pr-10 text-sm text-slate-200 placeholder:text-slate-500 focus:outline-none focus:border-indigo-500/50 focus:bg-white/10 transition-all duration-300 shadow-inner"
          />
          <button
            onClick={addTask}
            className="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-slate-400 hover:text-indigo-400 hover:bg-indigo-500/20 rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
          </button>
        </div>

        {/* Task List */}
        <div className="space-y-2">
          {isLoading && tasks.length === 0 && (
            <div className="text-center py-8 text-slate-500 text-sm">Loading…</div>
          )}
          <AnimatePresence>
            {!isLoading && tasks.length === 0 && !error && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="text-center py-8 text-slate-500 text-sm italic"
              >
                No tasks yet. Add one above!
              </motion.div>
            )}
            {tasks.map(task => (
              <motion.div
                key={task.id}
                initial={{ opacity: 0, y: 10, scale: 0.95 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9, x: -10 }}
                transition={{ duration: 0.2 }}
                className="group flex items-center gap-3 p-3 bg-white/5 border border-white/5 hover:border-white/10 hover:bg-white/10 rounded-xl transition-all duration-200"
              >
                {/* Custom Checkbox */}
                <button
                  onClick={() => toggleTask(task.id)}
                  className={`flex-shrink-0 w-5 h-5 rounded border flex items-center justify-center transition-all duration-300 ${
                    task.completed
                      ? 'bg-indigo-500 border-indigo-500'
                      : 'border-slate-500 hover:border-indigo-400 bg-white/5'
                  }`}
                >
                  {task.completed && <Check className="w-3.5 h-3.5 text-white" />}
                </button>

                {/* Task Title with optional Time Badge */}
                {(() => {
                  const { tag, title } = parseTaskTitle(task.title);
                  return (
                    <div className="flex-1 flex items-center gap-2 min-w-0">
                      {tag && (
                        <span className={`text-[10px] font-mono font-medium px-1.5 py-0.5 rounded border transition-colors flex-shrink-0 ${
                          task.completed
                            ? 'bg-slate-500/10 text-slate-500 border-slate-500/20'
                            : 'bg-indigo-500/15 text-indigo-300 border-indigo-500/30'
                        }`}>
                          {tag}
                        </span>
                      )}
                      <span className={`text-sm truncate transition-all duration-300 ${
                        task.completed ? 'text-slate-500 line-through' : 'text-slate-200'
                      }`}>
                        {title}
                      </span>
                    </div>
                  );
                })()}

                {/* Focus / Play Button */}
                {onFocusTask && !task.completed && (
                  <button
                    onClick={() => onFocusTask(task.title)}
                    title="Focus on this task"
                    className="opacity-0 group-hover:opacity-100 p-1.5 text-slate-400 hover:text-indigo-400 hover:bg-indigo-500/20 rounded-lg transition-all duration-200"
                  >
                    <Play className="w-3.5 h-3.5 fill-current" />
                  </button>
                )}

                {/* Delete Button */}
                <button
                  onClick={() => deleteTask(task.id)}
                  className="opacity-0 group-hover:opacity-100 p-1.5 text-slate-500 hover:text-rose-400 hover:bg-rose-500/20 rounded-lg transition-all duration-200"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
