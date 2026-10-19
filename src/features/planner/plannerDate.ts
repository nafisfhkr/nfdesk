export function shiftLocalDate(date: string, days: number): string {
  const [year, month, day] = date.split('-').map(Number);
  const local = new Date(year, month - 1, day, 12, 0, 0);
  local.setDate(local.getDate() + days);
  return [local.getFullYear(), String(local.getMonth() + 1).padStart(2, '0'), String(local.getDate()).padStart(2, '0')].join('-');
}

export function formatPlannerDate(date: string): string {
  const [year, month, day] = date.split('-').map(Number);
  return new Intl.DateTimeFormat('id-ID', {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(new Date(year, month - 1, day, 12, 0, 0));
}
