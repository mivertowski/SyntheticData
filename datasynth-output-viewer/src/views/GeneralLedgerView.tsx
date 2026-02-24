import { useEffect, useState, useMemo } from 'react';
import { loadJournalEntriesCsv, loadChartOfAccounts } from '../api/data';
import { DataTable } from '../components/DataTable';
import type { JournalEntryRow } from '../types';
import './GeneralLedgerView.css';

function formatNum(v: unknown): string {
  if (v == null || v === '') return '';
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return String(v);
  return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

const GL_COLUMNS = [
  { key: 'posting_date', label: 'Date', width: '100px' },
  { key: 'document_id', label: 'Document ID', width: '140px' },
  { key: 'document_type', label: 'Type', width: '100px' },
  { key: 'reference', label: 'Reference', width: '120px' },
  { key: 'auxiliary_account_number', label: 'Compte aux.', width: '90px' },
  { key: 'auxiliary_account_label', label: 'Libellé aux.', width: '140px' },
  { key: 'debit_amount', label: 'Debit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'credit_amount', label: 'Credit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'lettrage', label: 'Lettrage', width: '70px' },
  { key: 'lettrage_date', label: 'Date lettrage', width: '100px' },
  { key: 'libelle_ecriture', label: 'Libellé écriture', width: '220px' },
  { key: 'is_fraud', label: 'Fraud', width: '60px' },
];

export function GeneralLedgerView() {
  const [allRows, setAllRows] = useState<JournalEntryRow[]>([]);
  const [coaMap, setCoaMap] = useState<Map<string, { name: string; category: string }>>(new Map());
  const [selectedAccount, setSelectedAccount] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([loadJournalEntriesCsv(), loadChartOfAccounts()])
      .then(([data, coa]) => {
        setAllRows(data);
        setCoaMap(coa);
        setError(null);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load journal entries');
        setAllRows([]);
      })
      .finally(() => setLoading(false));
  }, []);

  const selectedAccountName = selectedAccount ? (coaMap.get(selectedAccount)?.name ?? null) : null;

  const accountList = useMemo(() => {
    const set = new Set<string>();
    allRows.forEach((r) => set.add(r.gl_account));
    return Array.from(set).sort();
  }, [allRows]);

  const glRows = useMemo(() => {
    if (!selectedAccount) return [];
    return allRows
      .filter((r) => r.gl_account === selectedAccount)
      .map((r, i) => {
        const libelle_ecriture =
          (r.line_text != null && String(r.line_text).trim() !== '')
            ? String(r.line_text)
            : (r.header_text != null && String(r.header_text).trim() !== '')
              ? String(r.header_text)
              : '';
        return { ...r, _rowKey: `${r.document_id}-${r.line_number ?? i}`, libelle_ecriture };
      });
  }, [allRows, selectedAccount]);

  useEffect(() => {
    if (accountList.length > 0 && !selectedAccount) setSelectedAccount(accountList[0]);
  }, [accountList, selectedAccount]);

  if (loading) return <div className="general-ledger-view loading">Loading general ledger data…</div>;
  if (error && allRows.length === 0) return <div className="general-ledger-view error">Error: {error}</div>;

  if (allRows.length === 0) {
    return (
      <div className="general-ledger-view">
        <h2>General Ledger</h2>
        <p className="general-ledger-empty">No journal entries in output. Run generation first.</p>
      </div>
    );
  }

  return (
    <div className="general-ledger-view">
      <h2>General Ledger</h2>
      <p className="general-ledger-desc">Transactions per GL account from <code>journal_entries.csv</code>. Select an account to view its transactions.</p>
      <div className="general-ledger-selector">
        <label htmlFor="gl-account">GL Account:</label>
        <select
          id="gl-account"
          value={selectedAccount}
          onChange={(e) => setSelectedAccount(e.target.value)}
        >
          {accountList.map((acct) => {
            const name = coaMap.get(acct)?.name;
            const label = name ? `${acct} — ${name}` : acct;
            return (
              <option key={acct} value={acct}>
                {label}
              </option>
            );
          })}
        </select>
        {selectedAccountName != null && (
          <span className="general-ledger-account-name" title={selectedAccountName}>
            {selectedAccountName}
          </span>
        )}
        <span className="general-ledger-count">{glRows.length} line(s)</span>
      </div>
      <DataTable
        key={selectedAccount}
        data={glRows as unknown as Record<string, unknown>[]}
        columns={GL_COLUMNS}
        keyField="_rowKey"
        pageSize={100}
        maxHeight="65vh"
      />
    </div>
  );
}
