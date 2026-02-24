import { useState, useEffect } from 'react';
import { getDataBase, setDataBase, DATA_BASE } from './config';
import { JECView } from './views/JECView';
import { MasterDataView } from './views/MasterDataView';
import { FraudAnomalyView } from './views/FraudAnomalyView';
import { TrialBalanceView } from './views/TrialBalanceView';
import { GeneralLedgerView } from './views/GeneralLedgerView';
import { AuxiliaryLedgerView } from './views/AuxiliaryLedgerView';
import { SubledgerView } from './views/SubledgerView';
import { GraphView } from './views/GraphView';
import './App.css';

const THEME_STORAGE_KEY = 'datasynth-viewer-theme';

export type ThemeId = 'light' | 'dark' | 'system';

function loadTheme(): ThemeId {
  try {
    const s = localStorage.getItem(THEME_STORAGE_KEY);
    if (s === 'light' || s === 'dark' || s === 'system') return s;
  } catch {
    /* ignore */
  }
  return 'system';
}

function getSystemTheme(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
}

function applyTheme(theme: ThemeId) {
  const root = document.documentElement;
  const resolved = theme === 'system' ? getSystemTheme() : theme;
  root.setAttribute('data-theme', resolved);
}

type TabId = 'jec' | 'master' | 'fraud' | 'trial' | 'gl' | 'auxiliary' | 'subledger' | 'graph';

const TABS: { id: TabId; label: string }[] = [
  { id: 'jec', label: 'Journal (JEC / FEC)' },
  { id: 'master', label: 'Master Data' },
  { id: 'fraud', label: 'Fraud & Anomalies' },
  { id: 'trial', label: 'Trial Balance' },
  { id: 'gl', label: 'General Ledger' },
  { id: 'auxiliary', label: 'Auxiliary Ledger' },
  { id: 'subledger', label: 'Subledger' },
  { id: 'graph', label: 'Graph (Neo4j)' },
];

function App() {
  const [activeTab, setActiveTab] = useState<TabId>('jec');
  const [theme, setTheme] = useState<ThemeId>(loadTheme);
  const [dataBaseKey, setDataBaseKey] = useState(0);
  const [loadDataOpen, setLoadDataOpen] = useState(false);
  const [loadDataInput, setLoadDataInput] = useState('');

  useEffect(() => {
    applyTheme(theme);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, theme);
    } catch {
      /* ignore */
    }
    if (theme === 'system') {
      const m = window.matchMedia('(prefers-color-scheme: light)');
      const listener = () => applyTheme('system');
      m.addEventListener('change', listener);
      return () => m.removeEventListener('change', listener);
    }
  }, [theme]);

  const openLoadData = () => {
    setLoadDataInput(getDataBase());
    setLoadDataOpen(true);
  };

  useEffect(() => {
    if (!loadDataOpen) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setLoadDataOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [loadDataOpen]);

  const applyLoadData = () => {
    const value = loadDataInput.trim();
    setDataBase(value || DATA_BASE);
    setLoadDataOpen(false);
    setDataBaseKey((k) => k + 1);
  };

  const resetLoadData = () => {
    setDataBase(DATA_BASE);
    setLoadDataOpen(false);
    setDataBaseKey((k) => k + 1);
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="app-header-top">
          <div>
            <h1>DataSynth Output Viewer</h1>
            <p className="app-subtitle">Visualize generated journal entries, master data, trial balance, and anomalies</p>
          </div>
          <div className="app-header-actions">
            <button
              type="button"
              className="app-load-data-btn"
              onClick={openLoadData}
              aria-label="Load data from another location"
            >
              Load data
            </button>
            <div className="app-theme">
              <label htmlFor="app-theme-select">Theme</label>
              <select
                id="app-theme-select"
                className="app-theme-select"
                value={theme}
                onChange={(e) => setTheme(e.target.value as ThemeId)}
                aria-label="Theme"
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>
          </div>
        </div>
      </header>

      {loadDataOpen && (
        <div
          className="app-load-data-overlay"
          role="dialog"
          aria-modal="true"
          aria-labelledby="app-load-data-title"
          onClick={(e) => e.target === e.currentTarget && setLoadDataOpen(false)}
        >
          <div className="app-load-data-modal" onClick={(e) => e.stopPropagation()}>
            <h2 id="app-load-data-title">Load data from</h2>
            <p className="app-load-data-hint">
              Enter the base URL or path for the output directory (e.g. <code>/data</code> or <code>https://example.com/output</code>).
              Relative paths are resolved from the app origin.
            </p>
            <input
              type="text"
              className="app-load-data-input"
              value={loadDataInput}
              onChange={(e) => setLoadDataInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && applyLoadData()}
              placeholder={DATA_BASE}
              aria-label="Data base URL or path"
            />
            <div className="app-load-data-actions">
              <button type="button" className="app-load-data-btn-cancel" onClick={() => setLoadDataOpen(false)}>
                Cancel
              </button>
              <button type="button" className="app-load-data-btn-reset" onClick={resetLoadData}>
                Reset to default
              </button>
              <button type="button" className="app-load-data-btn-apply" onClick={applyLoadData}>
                Load
              </button>
            </div>
          </div>
        </div>
      )}
      <nav className="app-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            type="button"
            className={activeTab === tab.id ? 'active' : ''}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>
      <main className="app-main" key={dataBaseKey}>
        {activeTab === 'jec' && <JECView />}
        {activeTab === 'master' && <MasterDataView />}
        {activeTab === 'fraud' && <FraudAnomalyView />}
        {activeTab === 'trial' && <TrialBalanceView />}
        {activeTab === 'gl' && <GeneralLedgerView />}
        {activeTab === 'auxiliary' && <AuxiliaryLedgerView />}
        {activeTab === 'subledger' && <SubledgerView />}
        {activeTab === 'graph' && (
          <GraphView
            resolvedTheme={theme === 'system' ? getSystemTheme() : theme}
          />
        )}
      </main>
    </div>
  );
}

export default App;
