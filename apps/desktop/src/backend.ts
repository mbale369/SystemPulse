import { invoke, isTauri } from '@tauri-apps/api/core';
import { fixture } from './fixtures';
import { validateSnapshot, validateHealthReport, type Snapshot } from './model';
export const demoMode = !isTauri();
let sequence = 0;
export async function snapshot(): Promise<Snapshot> {
  const value: unknown = demoMode ? fixture(++sequence) : await invoke('get_snapshot');
  return validateSnapshot(value);
}
export async function configure(profile: string): Promise<void> {
  if (!['eco','balanced','detailed'].includes(profile)) throw new Error('Invalid profile');
  if (!demoMode) await invoke('set_profile', { profile });
}
export async function health(): Promise<import('./model').HealthReport> {
  if (demoMode) return {state:'unknown',alerts:[],resolved:[],missing_evidence:['Données de démonstration / Simulated data']};
  return validateHealthReport(await invoke<unknown>('get_health'));
}
