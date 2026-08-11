import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Check, Trash2, Plus } from 'lucide-react';

type Task = { id: string; title: string; completed: boolean; createdAt: number };

export default function TasksView() {
  const [tasks, setTasks] = useState<Task[]>(() => {
    const saved = localStorage.getItem('nfdesk-tasks');
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (e) {
        return [];
      }
    }
    return [];
  });
  const [newTaskTitle, setNewTaskTitle] = useState('');

  useEffect(() => {
    localStorage.setItem('nfdesk-tasks', JSON.stringify(tasks));
  }, [tasks]);

  const addTask = () => {
    if (newTaskTitle.trim() === '') return;
    const newTask: Task = {
      id: crypto.randomUUID(),
      title: newTaskTitle.trim(),
      completed: false,
      createdAt: Date.now(),
    };
    setTasks([...tasks, newTask]);
    setNewTaskTitle('');
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      addTask();
    }
  };

  const toggleTask = (id: string) => {
    setTasks(tasks.map(t => t.id === id ? { ...t, completed: !t.completed } : t));
  };

  const deleteTask = (id: string) => {
    setTasks(tasks.filter(t => t.id !== id));
  };

  return (
    <div className="flex flex-col h-full overflow-hidden text-slate-200">
      <div className="flex-1 overflow-y-auto pr-1 pb-4 custom-scrollbar">
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
          <AnimatePresence>
            {tasks.length === 0 && (
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
                
                {/* Task Title */}
                <span className={`flex-1 text-sm truncate transition-all duration-300 ${task.completed ? 'text-slate-500 line-through' : 'text-slate-200'}`}>
                  {task.title}
                </span>

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
