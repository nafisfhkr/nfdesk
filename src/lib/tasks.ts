import { invoke } from '@tauri-apps/api/core';

export type TaskStatus = 'planned' | 'in_progress' | 'completed' | 'cancelled';
export type TaskPriority = 'low' | 'medium' | 'high';

export interface Task {
  id: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: TaskPriority | null;
  planned_date: string;
  scheduled_at: string | null;
  deadline_at: string | null;
  estimated_sessions: number | null;
  estimated_session_minutes: number | null;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
  metadata_extensions?: Record<string, unknown>;
}

export interface TaskCreateRequest {
  title: string;
  planned_date: string;
  description?: string | null;
  priority?: TaskPriority | null;
  scheduled_at?: string | null;
  deadline_at?: string | null;
  estimated_sessions?: number | null;
  estimated_session_minutes?: number | null;
}

export interface TaskPatch {
  title?: string;
  description?: string | null;
  priority?: TaskPriority | null;
  scheduled_at?: string | null;
  deadline_at?: string | null;
  estimated_sessions?: number | null;
  estimated_session_minutes?: number | null;
}

export interface TaskListResponse {
  tasks: Task[];
  date: string;
  migration_preview_available: boolean;
}

export interface LegacyTaskPreviewItem {
  proposed_id: string;
  title: string;
  status: TaskStatus;
  planned_date: string;
  previewed_at: string;
  original_line: string;
}

export interface LegacyTaskMigrationPreview {
  found: boolean;
  source_kind: string;
  date: string;
  items: LegacyTaskPreviewItem[];
  ignored_line_count: number;
}

export function todayLocalDate(): string {
  const d = new Date();
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const dd = String(d.getDate()).padStart(2, '0');
  return `${d.getFullYear()}-${mm}-${dd}`;
}

export async function listTasksForDate(date: string): Promise<TaskListResponse> {
  return invoke<TaskListResponse>('tasks_list_for_date', { date });
}

export async function createTask(task: TaskCreateRequest): Promise<Task> {
  return invoke<Task>('task_create', { task });
}

export async function updateTask(
  taskId: string,
  plannedDate: string,
  patch: TaskPatch
): Promise<Task> {
  return invoke<Task>('task_update', {
    task_id: taskId,
    planned_date: plannedDate,
    patch,
  });
}

export async function completeTask(taskId: string, plannedDate: string): Promise<Task> {
  return invoke<Task>('task_complete', {
    task_id: taskId,
    planned_date: plannedDate,
  });
}

export async function cancelTask(taskId: string, plannedDate: string): Promise<Task> {
  return invoke<Task>('task_cancel', {
    task_id: taskId,
    planned_date: plannedDate,
  });
}

export async function rescheduleTask(
  taskId: string,
  sourceDate: string,
  targetDate: string
): Promise<Task> {
  return invoke<Task>('task_reschedule', {
    task_id: taskId,
    source_date: sourceDate,
    target_date: targetDate,
  });
}

export async function carryTaskToToday(taskId: string, sourceDate: string): Promise<Task> {
  return invoke<Task>('task_carry_to_today', {
    task_id: taskId,
    source_date: sourceDate,
  });
}

export async function previewLegacyTasks(date: string): Promise<LegacyTaskMigrationPreview> {
  return invoke<LegacyTaskMigrationPreview>('task_legacy_preview', { date });
}
