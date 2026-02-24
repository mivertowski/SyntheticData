const DATA_BASE_DEFAULT = (import.meta.env.VITE_DATA_BASE as string) || '/data';
const STORAGE_KEY = 'datasynth-viewer-data-base';

function getStoredBase(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

/** Mutable base URL; can be changed at runtime via setDataBase (e.g. Load data button). */
let currentDataBase = getStoredBase() || DATA_BASE_DEFAULT;

/** Default base (from env or /data). Used when resetting to default. */
export const DATA_BASE = DATA_BASE_DEFAULT;

/** Current base URL for data loading (may have been overridden by user). */
export function getDataBase(): string {
  return currentDataBase;
}

/** Set data base URL and persist. Call when user chooses a different folder/source. */
export function setDataBase(url: string): void {
  const base = (url || DATA_BASE_DEFAULT).trim() || DATA_BASE_DEFAULT;
  currentDataBase = base.replace(/\/$/, '');
  try {
    if (currentDataBase === DATA_BASE_DEFAULT) {
      localStorage.removeItem(STORAGE_KEY);
    } else {
      localStorage.setItem(STORAGE_KEY, currentDataBase);
    }
  } catch {
    /* ignore */
  }
}

export function dataUrl(path: string): string {
  const base = currentDataBase.replace(/\/$/, '');
  const p = path.startsWith('/') ? path : `/${path}`;
  return `${base}${p}`;
}
