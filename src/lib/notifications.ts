import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

/**
 * Synthesizes a soft, professional harmonic chime using Web Audio API.
 * Requires 0 external audio files and incurs 0 MB memory overhead.
 */
export function playChime(type: 'FOCUS' | 'BREAK' = 'FOCUS') {
  try {
    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) return;

    const ctx = new AudioCtx();
    const now = ctx.currentTime;

    // Frequencies: Focus completed uses D5 -> A5 (pleasant harmonic fifth).
    // Break completed uses E5 -> B5 (refreshing tone).
    const [freq1, freq2] = type === 'FOCUS' ? [587.33, 880.0] : [659.25, 987.77];

    // First tone
    const osc1 = ctx.createOscillator();
    const gain1 = ctx.createGain();
    osc1.type = 'sine';
    osc1.frequency.setValueAtTime(freq1, now);
    gain1.gain.setValueAtTime(0.15, now);
    gain1.gain.exponentialRampToValueAtTime(0.001, now + 0.6);
    osc1.connect(gain1);
    gain1.connect(ctx.destination);
    osc1.start(now);
    osc1.stop(now + 0.6);

    // Second tone (slightly delayed for gentle arpeggio chime effect)
    const osc2 = ctx.createOscillator();
    const gain2 = ctx.createGain();
    osc2.type = 'sine';
    osc2.frequency.setValueAtTime(freq2, now + 0.15);
    gain2.gain.setValueAtTime(0.15, now + 0.15);
    gain2.gain.exponentialRampToValueAtTime(0.001, now + 0.9);
    osc2.connect(gain2);
    gain2.connect(ctx.destination);
    osc2.start(now + 0.15);
    osc2.stop(now + 0.9);

    // Clean up AudioContext after sound finishes
    setTimeout(() => {
      ctx.close().catch(() => {});
    }, 1200);
  } catch (err) {
    console.warn('Audio chime failed to play:', err);
  }
}

/**
 * Triggers native Windows desktop notification + audio chime.
 */
export async function notifyTimerCompletion(
  mode: 'FOCUS' | 'BREAK',
  taskTitle?: string
) {
  // 1. Always play the synthesized audio chime immediately
  playChime(mode);

  // 2. Dispatch native Windows toast notification
  try {
    let permissionGranted = await isPermissionGranted();
    if (!permissionGranted) {
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }

    if (permissionGranted) {
      if (mode === 'FOCUS') {
        const taskText = taskTitle ? ` on "${taskTitle}"` : '';
        sendNotification({
          title: '🎉 Focus Session Completed!',
          body: `Great job! Focus session${taskText} is finished. Time for a short break.`,
        });
      } else {
        sendNotification({
          title: '☕ Break Finished!',
          body: 'Break is over. Ready to start your next focus session?',
        });
      }
    }
  } catch (err) {
    console.warn('Desktop notification error:', err);
  }
}
