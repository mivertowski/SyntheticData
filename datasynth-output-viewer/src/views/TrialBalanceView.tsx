import { useEffect, useState, useMemo } from 'react';
import { loadJournalEntriesCsv, loadChartOfAccounts } from '../api/data';
import { DataTable } from '../components/DataTable';
import type { JournalEntryRow } from '../types';
import './TrialBalanceView.css';

function formatNum(v: unknown): string {
  if (v == null || v === '') return '';
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return String(v);
  return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

const TB_COLUMNS = [
  { key: 'account_code', label: 'Account', width: '100px' },
  { key: 'account_name', label: 'Account Name', width: '200px' },
  { key: 'category', label: 'Category', width: '100px' },
  { key: 'debit_balance', label: 'Debit', width: '120px', format: (v: unknown) => formatNum(v) },
  { key: 'credit_balance', label: 'Credit', width: '120px', format: (v: unknown) => formatNum(v) },
];

function parseNum(v: string | number): number {
  if (typeof v === 'number' && !Number.isNaN(v)) return v;
  const n = parseFloat(String(v).replace(/,/g, ''));
  return Number.isNaN(n) ? 0 : n;
}

/** Build trial balance from journal entry rows up to and including the given date (YYYY-MM-DD). */
function computeTrialBalance(
  rows: JournalEntryRow[],
  asOfDate: string,
  coaMap: Map<string, { name: string; category: string }>
): Array<{ account_code: string; account_name: string; category: string; debit_balance: number; credit_balance: number }> {
  const byAccount = new Map<string, { debit: number; credit: number }>();
  for (const r of rows) {
    const postDate = String(r.posting_date ?? '').slice(0, 10);
    if (postDate > asOfDate) continue;
    const acct = r.gl_account || '';
    if (!acct) continue;
    let agg = byAccount.get(acct);
    if (!agg) {
      agg = { debit: 0, credit: 0 };
      byAccount.set(acct, agg);
    }
    agg.debit += parseNum(r.debit_amount);
    agg.credit += parseNum(r.credit_amount);
  }
  const entries: Array<{ account_code: string; account_name: string; category: string; debit_balance: number; credit_balance: number }> = [];
  for (const [account_code, agg] of byAccount.entries()) {
    const debitBalance = Math.max(0, agg.debit - agg.credit);
    const creditBalance = Math.max(0, agg.credit - agg.debit);
    const info = coaMap.get(account_code);
    entries.push({
      account_code,
      account_name: info?.name ?? '—',
      category: info?.category ?? '—',
      debit_balance: debitBalance,
      credit_balance: creditBalance,
    });
  }
  entries.sort((a, b) => a.account_code.localeCompare(b.account_code, 'en', { numeric: true }));
  return entries;
}

/** Unique sorted dates from JE posting_date (YYYY-MM-DD). */
function uniqueDates(rows: JournalEntryRow[]): string[] {
  const set = new Set<string>();
  for (const r of rows) {
    const d = String(r.posting_date ?? '').slice(0, 10);
    if (d.match(/^\d{4}-\d{2}-\d{2}$/)) set.add(d);
  }
  return Array.from(set).sort();
}

export function TrialBalanceView() {
  const [jeRows, setJeRows] = useState<JournalEntryRow[]>([]);
  const [coaMap, setCoaMap] = useState<Map<string, { name: string; category: string }>>(new Map());
  const [selectedDate, setSelectedDate] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([loadJournalEntriesCsv(), loadChartOfAccounts()])
      .then(([rows, coa]) => {
        setJeRows(rows);
        setCoaMap(coa);
        const dates = uniqueDates(rows);
        setSelectedDate(dates.length > 0 ? dates[dates.length - 1] : '');
        setError(null);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load journal entries');
        setJeRows([]);
        setSelectedDate('');
      })
      .finally(() => setLoading(false));
  }, []);

  const dateOptions = useMemo(() => uniqueDates(jeRows), [jeRows]);

  const tbEntries = useMemo(
    () => (selectedDate ? computeTrialBalance(jeRows, selectedDate, coaMap) : []),
    [jeRows, selectedDate, coaMap]
  );

  if (loading) return <div className="trial-balance-view loading">Loading journal entries…</div>;
  if (error && jeRows.length === 0) return <div className="trial-balance-view error">Error: {error}</div>;

  if (jeRows.length === 0) {
    return (
      <div className="trial-balance-view">
        <h2>Trial Balance</h2>
        <p className="trial-balance-empty">No journal entries in output. Run generation first.</p>
      </div>
    );
  }

  return (
    <div className="trial-balance-view">
      <h2>Trial Balance</h2>
      <p className="trial-balance-desc">
        Computed on the fly from <code>journal_entries.csv</code> — cumulative balances as of the selected date.
      </p>
      <div className="trial-balance-selector">
        <label htmlFor="tb-date">As of date:</label>
        <select
          id="tb-date"
          value={selectedDate}
          onChange={(e) => setSelectedDate(e.target.value)}
        >
          {dateOptions.map((d) => (
            <option key={d} value={d}>
              {d}
            </option>
          ))}
        </select>
        <span className="trial-balance-meta">{tbEntries.length} account(s)</span>
      </div>
      <DataTable
        data={tbEntries as unknown as Record<string, unknown>[]}
        columns={TB_COLUMNS}
        keyField="account_code"
        pageSize={100}
        maxHeight="65vh"
      />
    </div>
  );
}
