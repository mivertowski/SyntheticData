import { useEffect, useState, useMemo } from 'react';
import { loadJournalEntriesCsv } from '../api/data';
import { DataTable } from '../components/DataTable';
import type { JournalEntryRow } from '../types';
import './AuxiliaryLedgerView.css';

function formatNum(v: unknown): string {
  if (v == null || v === '') return '';
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return String(v);
  return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

const AUX_GL_COLUMNS = [
  { key: 'posting_date', label: 'Date', width: '100px' },
  { key: 'document_id', label: 'Document ID', width: '140px' },
  { key: 'document_type', label: 'Type', width: '100px' },
  { key: 'gl_account', label: 'GL Account', width: '90px' },
  { key: 'reference', label: 'Reference', width: '120px' },
  { key: 'debit_amount', label: 'Debit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'credit_amount', label: 'Credit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'lettrage', label: 'Lettrage', width: '70px' },
  { key: 'lettrage_date', label: 'Date lettrage', width: '100px' },
  { key: 'libelle_ecriture', label: 'Libellé écriture', width: '220px' },
  { key: 'is_fraud', label: 'Fraud', width: '60px' },
];

export function AuxiliaryLedgerView() {
  const [allRows, setAllRows] = useState<JournalEntryRow[]>([]);
  const [selectedAux, setSelectedAux] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadJournalEntriesCsv()
      .then((data) => {
        setAllRows(data);
        setError(null);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load journal entries');
        setAllRows([]);
      })
      .finally(() => setLoading(false));
  }, []);

  const auxiliaryList = useMemo(() => {
    const seen = new Map<string, string>();
    allRows.forEach((r) => {
      const num = r.auxiliary_account_number?.trim();
      if (num) {
        const label = r.auxiliary_account_label?.trim() ?? num;
        if (!seen.has(num)) seen.set(num, label);
      }
    });
    return Array.from(seen.entries())
      .map(([num, label]) => ({ number: num, label }))
      .sort((a, b) => a.number.localeCompare(b.number));
  }, [allRows]);

  const auxRows = useMemo(() => {
    if (!selectedAux) return [];
    return allRows
      .filter((r) => (r.auxiliary_account_number ?? '').trim() === selectedAux)
      .map((r, i) => {
        const libelle_ecriture =
          (r.line_text != null && String(r.line_text).trim() !== '')
            ? String(r.line_text)
            : (r.header_text != null && String(r.header_text).trim() !== '')
              ? String(r.header_text)
              : '';
        return { ...r, _rowKey: `${r.document_id}-${r.line_number ?? i}`, libelle_ecriture };
      });
  }, [allRows, selectedAux]);

  const selectedLabel = auxiliaryList.find((a) => a.number === selectedAux)?.label ?? selectedAux;

  useEffect(() => {
    if (auxiliaryList.length > 0 && !selectedAux) setSelectedAux(auxiliaryList[0].number);
  }, [auxiliaryList, selectedAux]);

  if (loading) return <div className="auxiliary-ledger-view loading">Loading auxiliary ledger data…</div>;
  if (error && allRows.length === 0) return <div className="auxiliary-ledger-view error">Error: {error}</div>;

  if (allRows.length === 0) {
    return (
      <div className="auxiliary-ledger-view">
        <h2>Auxiliary Ledger</h2>
        <p className="auxiliary-ledger-empty">No journal entries in output. Run generation first.</p>
      </div>
    );
  }

  if (auxiliaryList.length === 0) {
    return (
      <div className="auxiliary-ledger-view">
        <h2>Auxiliary Ledger</h2>
        <p className="auxiliary-ledger-empty">
          No auxiliary accounts in this dataset. Auxiliary accounts (e.g. 401xxxx, 411xxxx) are present when French GAAP is enabled and document-flow JEs are generated.
        </p>
      </div>
    );
  }

  return (
    <div className="auxiliary-ledger-view">
      <h2>Auxiliary Ledger</h2>
      <p className="auxiliary-ledger-desc">
        Transactions per <strong>auxiliary account</strong> (comptes auxiliaires, e.g. 401xxxx vendors, 411xxxx customers) from <code>journal_entries.csv</code>. Select an auxiliary account to view its lines.
      </p>
      <div className="auxiliary-ledger-selector">
        <label htmlFor="aux-account">Auxiliary account:</label>
        <select
          id="aux-account"
          value={selectedAux}
          onChange={(e) => setSelectedAux(e.target.value)}
        >
          {auxiliaryList.map(({ number, label }) => (
            <option key={number} value={number}>
              {number} — {label}
            </option>
          ))}
        </select>
        {selectedLabel && selectedLabel !== selectedAux && (
          <span className="auxiliary-ledger-account-label" title={selectedLabel}>
            {selectedLabel}
          </span>
        )}
        <span className="auxiliary-ledger-count">{auxRows.length} line(s)</span>
      </div>
      <DataTable
        data={auxRows as unknown as Record<string, unknown>[]}
        columns={AUX_GL_COLUMNS}
        keyField="_rowKey"
        pageSize={100}
        maxHeight="65vh"
      />
    </div>
  );
}
