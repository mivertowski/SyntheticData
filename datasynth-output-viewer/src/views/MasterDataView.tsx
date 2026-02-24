import { useEffect, useState } from 'react';
import { loadMasterData } from '../api/data';
import { DataTable } from '../components/DataTable';
import type { MasterRecord } from '../types';
import './MasterDataView.css';

type MasterTab = 'vendors' | 'customers' | 'materials' | 'fixed_assets' | 'employees';

interface MasterState {
  vendors: MasterRecord[];
  customers: MasterRecord[];
  materials: MasterRecord[];
  fixed_assets: MasterRecord[];
  employees: MasterRecord[];
}

const TABS: { id: MasterTab; label: string }[] = [
  { id: 'vendors', label: 'Vendors' },
  { id: 'customers', label: 'Customers' },
  { id: 'materials', label: 'Materials' },
  { id: 'fixed_assets', label: 'Fixed Assets' },
  { id: 'employees', label: 'Employees' },
];

function columnsFromSample(rows: MasterRecord[], maxCols = 12): { key: string; label: string }[] {
  if (rows.length === 0) return [];
  const keys = Object.keys(rows[0]).filter((k) => typeof rows[0][k] !== 'object' || rows[0][k] === null);
  return keys.slice(0, maxCols).map((k) => ({ key: k, label: k.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()) }));
}

/** For vendors/customers under French GAAP: show auxiliary_gl_account (401xxxx/411xxxx) in the Account number column. */
function normalizeVendorCustomerRows(rows: MasterRecord[], tab: MasterTab): MasterRecord[] {
  if (rows.length === 0 || (tab !== 'vendors' && tab !== 'customers')) return rows;
  return rows.map((row) => {
    const r = { ...row } as Record<string, unknown>;
    const aux = r.auxiliary_gl_account;
    if (aux != null && aux !== '') {
      r.account_number = aux;
    }
    return r as MasterRecord;
  });
}

export function MasterDataView() {
  const [data, setData] = useState<MasterState | null>(null);
  const [activeTab, setActiveTab] = useState<MasterTab>('vendors');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadMasterData()
      .then((d) => {
        setData((d ?? { vendors: [], customers: [], materials: [], fixed_assets: [], employees: [] }) as MasterState);
        setError(null);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load master data');
        setData({ vendors: [], customers: [], materials: [], fixed_assets: [], employees: [] });
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="master-data-view loading">Loading master data…</div>;
  if (error && !data) return <div className="master-data-view error">Error: {error}</div>;

  const state = data!;
  const rawRows = state[activeTab] as MasterRecord[];
  const rows = normalizeVendorCustomerRows(rawRows, activeTab);
  const cols = columnsFromSample(rows);

  return (
    <div className="master-data-view">
      <h2>Master Data</h2>
      <p className="master-data-desc">Detailed view of vendors, customers, materials, fixed assets, and employees from <code>master_data/</code>.</p>
      <div className="master-data-tabs">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            type="button"
            className={activeTab === tab.id ? 'active' : ''}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label} ({(state[tab.id] as MasterRecord[])?.length ?? 0})
          </button>
        ))}
      </div>
      {rows.length === 0 ? (
        <p className="master-data-empty">No {activeTab.replace('_', ' ')} data in output.</p>
      ) : (
        <DataTable
          data={rows}
          columns={cols.map((c) => ({ ...c, format: (v) => (v != null && typeof v === 'object' ? JSON.stringify(v) : String(v ?? '')) }))}
          keyField={cols[0]?.key ?? 'id'}
          pageSize={50}
          maxHeight="65vh"
        />
      )}
    </div>
  );
}
