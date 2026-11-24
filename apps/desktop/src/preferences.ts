import type { Language } from './i18n.ts';

export type Theme = 'dark' | 'light';
export interface Preferences {
  language: Language | 'system';
  theme: Theme | 'system';
}
export interface PreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}
export function loadPreferences(storage?: PreferenceStorage): Preferences {
  const read = (key: string): string | null => {
    try { return storage?.getItem(key) ?? null; } catch { return null; }
  };
  const language = read('language'), theme = read('theme');
  return {
    language: language === 'fr' || language === 'en' ? language : 'system',
    theme: theme === 'dark' || theme === 'light' ? theme : 'system',
  };
}
export function savePreferences(preferences: Preferences, storage?: PreferenceStorage): void {
  for (const key of ['language', 'theme'] as const) {
    try { storage?.setItem(key, preferences[key]); } catch { /* Retain the in-memory choice. */ }
  }
}
export function resolveLanguage(preference: Preferences['language'], languages: readonly string[]): Language {
  if (preference !== 'system') return preference;
  for (const locale of languages) {
    const primary = locale.toLowerCase().split(/[-_]/)[0];
    if (primary === 'fr' || primary === 'en') return primary;
  }
  // French is the existing product fallback; other languages are not translated.
  return 'fr';
}
export function resolveTheme(preference: Preferences['theme'], systemDark: boolean): Theme {
  return preference === 'system' ? (systemDark ? 'dark' : 'light') : preference;
}
