import { useEffect, useState, useMemo } from 'react';
import { loadJournalEntriesCsv, loadFEC, hasFEC } from '../api/data';
import { DataTable } from '../components/DataTable';
import type { JournalEntryRow, FECRow } from '../types';
import './JECView.css';

/** Fields to search in standard JE view */
const JE_SEARCH_KEYS: (keyof JournalEntryRow)[] = [
  'document_id',
  'company_code',
  'posting_date',
  'document_type',
  'gl_account',
  'auxiliary_account_number',
  'auxiliary_account_label',
  'lettrage',
  'lettrage_date',
  'reference',
  'header_text',
  'line_text',
];

/** Keys to search in FEC view (FECRow keys) */
const FEC_SEARCH_KEYS = [
  "Code journal",
  "Libellé journal",
  "Numéro de l'écriture",
  'Date de comptabilisation',
  'Numéro de compte',
  'Libellé de compte',
  "Numéro de compte auxiliaire",
  "Libellé de compte auxiliaire",
  "Référence de la pièce justificative",
  "Libellé de l'écriture comptable",
  'Lettrage',
  'Date de lettrage',
  "Identifiant de la devise",
] as const;

function matchesSearch<T extends Record<string, unknown>>(
  row: T,
  search: string,
  keys: readonly (keyof T)[]
): boolean {
  const q = search.trim().toLowerCase();
  if (!q) return true;
  for (const key of keys) {
    const val = row[key];
    if (val != null && String(val).toLowerCase().includes(q)) return true;
  }
  return false;
}

const JE_COLUMNS = [
  { key: 'document_id', label: 'Document ID', width: '140px' },
  { key: 'company_code', label: 'Company', width: '80px' },
  { key: 'posting_date', label: 'Posting Date', width: '100px' },
  { key: 'document_type', label: 'Type', width: '100px' },
  { key: 'gl_account', label: 'GL Account', width: '90px' },
  { key: 'auxiliary_account_number', label: 'Compte aux.', width: '90px' },
  { key: 'auxiliary_account_label', label: 'Libellé aux.', width: '140px' },
  { key: 'debit_amount', label: 'Debit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'credit_amount', label: 'Credit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'lettrage', label: 'Lettrage', width: '70px' },
  { key: 'lettrage_date', label: 'Date lettrage', width: '100px' },
  { key: 'reference', label: 'Reference', width: '120px' },
  { key: 'line_text', label: 'Line Text', width: '200px' },
  { key: 'is_fraud', label: 'Fraud', width: '60px' },
  { key: 'is_anomaly', label: 'Anomaly', width: '70px' },
];

const FEC_COLUMNS = [
  { key: "Code journal", label: "Code journal", width: '80px' },
  { key: "Libellé journal", label: "Libellé journal", width: '180px' },
  { key: "Numéro de l'écriture", label: "N° Écriture", width: '90px' },
  { key: 'Date de comptabilisation', label: 'Date comptab.', width: '110px' },
  { key: 'Numéro de compte', label: 'Compte', width: '90px' },
  { key: 'Libellé de compte', label: 'Libellé compte', width: '120px' },
  { key: "Numéro de compte auxiliaire", label: 'Compte aux.', width: '90px' },
  { key: "Libellé de compte auxiliaire", label: 'Libellé aux.', width: '140px' },
  { key: 'Montant au débit', label: 'Débit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'Montant au crédit', label: 'Crédit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'Lettrage', label: 'Lettrage', width: '70px' },
  { key: 'Date de lettrage', label: 'Date lettrage', width: '100px' },
  { key: "Référence de la pièce justificative", label: 'Réf. pièce', width: '140px' },
  { key: "Libellé de l'écriture comptable", label: 'Libellé écriture', width: '200px' },
  { key: "Identifiant de la devise", label: 'Devise', width: '70px' },
];

function formatNum(v: unknown): string {
  if (v == null || v === '') return '';
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return String(v);
  return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

export function JECView() {
  const [jeData, setJeData] = useState<JournalEntryRow[]>([]);
  const [fecData, setFecData] = useState<FECRow[] | null>(null);
  const [fecAvailable, setFecAvailable] = useState(false);
  const [viewMode, setViewMode] = useState<'je' | 'fec'>('je');
  const [search, setSearch] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const filteredJeData = useMemo(() => {
    if (!search.trim()) return jeData;
    return jeData.filter((row) => matchesSearch(row, search, JE_SEARCH_KEYS));
  }, [jeData, search]);

  const filteredFecData = useMemo(() => {
    if (!fecData) return null;
    if (!search.trim()) return fecData;
    return fecData.filter((row) =>
      matchesSearch(row as Record<string, unknown>, search, FEC_SEARCH_KEYS as (keyof FECRow)[])
    );
  }, [fecData, search]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    Promise.all([
      loadJournalEntriesCsv().catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : 'Failed to load journal entries');
        return [] as JournalEntryRow[];
      }),
      hasFEC(),
      loadFEC(),
    ]).then(([je, hasFec, fec]) => {
      if (cancelled) return;
      setJeData(je);
      setFecAvailable(hasFec && fec != null && fec.length > 0);
      setFecData(fec);
      setLoading(false);
    });
    return () => { cancelled = true; };
  }, []);

  if (loading) return <div className="jec-view loading">Loading journal entries…</div>;
  if (error && jeData.length === 0) return <div className="jec-view error">Error: {error}</div>;

  return (
    <div className="jec-view">
      <div className="jec-view-header">
        <h2>Journal des écritures comptables (JEC)</h2>
        {fecAvailable && (
          <div className="jec-view-toggle">
            <button
              type="button"
              className={viewMode === 'je' ? 'active' : ''}
              onClick={() => setViewMode('je')}
            >
              Standard (CSV)
            </button>
            <button
              type="button"
              className={viewMode === 'fec' ? 'active' : ''}
              onClick={() => setViewMode('fec')}
            >
              FEC (Fichier des Écritures Comptables)
            </button>
          </div>
        )}
      </div>
      {viewMode === 'je' && (
        <>
          <p className="jec-view-desc">Flat journal entry lines from <code>journal_entries.csv</code>. When generated with French GAAP, the <strong>GL Account</strong> column uses the Plan Comptable Général (PCG) chart of accounts.</p>
          <div className="jec-view-search-wrap">
            <label htmlFor="jec-search" className="jec-view-search-label">
              Search
            </label>
            <input
              id="jec-search"
              type="search"
              className="jec-view-search"
              placeholder="Lettrage, document ID, reference, libellé, compte aux., …"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              aria-label="Search by lettrage, document ID, reference, or other fields"
            />
            {search.trim() && (
              <span className="jec-view-search-count">
                {filteredJeData.length} / {jeData.length} rows
              </span>
            )}
          </div>
          <DataTable
            key={`je-${search.trim()}`}
            data={filteredJeData as unknown as Record<string, unknown>[]}
            columns={JE_COLUMNS}
            keyField="document_id"
            pageSize={100}
            maxHeight="65vh"
          />
        </>
      )}
      {viewMode === 'fec' && fecData && (
        <>
          <p className="jec-view-desc">French GAAP export (18 columns, Article A47 A-1 LPF) from <code>fec.csv</code>. <strong>Numéro de compte</strong> and <strong>Compte auxiliaire</strong> follow the Plan Comptable Général (PCG) chart of accounts.</p>
          <div className="jec-view-search-wrap">
            <label htmlFor="jec-search-fec" className="jec-view-search-label">
              Search
            </label>
            <input
              id="jec-search-fec"
              type="search"
              className="jec-view-search"
              placeholder="Lettrage, référence, libellé, compte, …"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              aria-label="Search by lettrage, reference, libellé, or other FEC fields"
            />
            {search.trim() && filteredFecData && (
              <span className="jec-view-search-count">
                {filteredFecData.length} / {fecData.length} rows
              </span>
            )}
          </div>
          <DataTable
            key={`fec-${search.trim()}`}
            data={(filteredFecData ?? fecData) as unknown as Record<string, unknown>[]}
            columns={FEC_COLUMNS}
            keyField="Numéro de l'écriture"
            pageSize={100}
            maxHeight="65vh"
          />
        </>
      )}
      {viewMode === 'fec' && !fecData?.length && fecAvailable === false && (
        <p className="jec-view-no-fec">No FEC file present. Generate with French GAAP / FEC export to see this view.</p>
      )}
    </div>
  );
}
