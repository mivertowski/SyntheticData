import { useEffect, useState, useMemo } from 'react';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  Legend,
} from 'recharts';
import { loadAnomalyLabels, loadFraudLabels, loadJournalEntriesCsv } from '../api/data';
import { DataTable } from '../components/DataTable';
import type { AnomalyLabel, JournalEntryRow } from '../types';
import './FraudAnomalyView.css';

const TABLE_COLUMNS = [
  { key: 'anomaly_id', label: 'ID', width: '120px' },
  { key: 'anomaly_category', label: 'Category', width: '100px' },
  { key: 'anomaly_type', label: 'Type', width: '140px' },
  { key: 'document_id', label: 'Document ID', width: '140px' },
  { key: 'company_code', label: 'Company', width: '80px' },
  { key: 'anomaly_date', label: 'Date', width: '100px' },
  { key: 'severity', label: 'Severity', width: '70px' },
  { key: 'scenario_display', label: 'Scenario / Scheme', width: '140px' },
  { key: 'description', label: 'Description', width: '220px' },
  { key: 'monetary_impact', label: 'Impact', width: '100px' },
];

/** All columns for the concerned-transaction subtable (full journal entry line). */
const CONCERNED_FULL_COLUMNS = [
  { key: 'line_number', label: '#', width: '44px' },
  { key: 'document_id', label: 'Document ID', width: '130px' },
  { key: 'company_code', label: 'Company', width: '80px' },
  { key: 'fiscal_year', label: 'FY', width: '44px' },
  { key: 'fiscal_period', label: 'Period', width: '56px' },
  { key: 'posting_date', label: 'Posting Date', width: '100px' },
  { key: 'document_date', label: 'Doc. Date', width: '100px' },
  { key: 'document_type', label: 'Type', width: '90px' },
  { key: 'gl_account', label: 'GL Account', width: '90px' },
  { key: 'auxiliary_account_number', label: 'Compte aux.', width: '90px' },
  { key: 'auxiliary_account_label', label: 'Libellé aux.', width: '130px' },
  { key: 'debit_amount', label: 'Debit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'credit_amount', label: 'Credit', width: '100px', format: (v: unknown) => formatNum(v) },
  { key: 'local_amount', label: 'Local', width: '90px', format: (v: unknown) => formatNum(v) },
  { key: 'currency', label: 'CCY', width: '50px' },
  { key: 'exchange_rate', label: 'Rate', width: '70px' },
  { key: 'reference', label: 'Reference', width: '110px' },
  { key: 'header_text', label: 'Header', width: '140px' },
  { key: 'cost_center', label: 'Cost Ctr', width: '80px' },
  { key: 'profit_center', label: 'Profit Ctr', width: '80px' },
  { key: 'line_text', label: 'Line Text', width: '200px' },
  { key: 'lettrage', label: 'Lettrage', width: '70px' },
  { key: 'lettrage_date', label: 'Date lettrage', width: '100px' },
  { key: 'created_by', label: 'Created By', width: '90px' },
  { key: 'source', label: 'Source', width: '80px' },
  { key: 'business_process', label: 'Process', width: '90px' },
  { key: 'ledger', label: 'Ledger', width: '70px' },
  { key: 'is_fraud', label: 'Fraud', width: '56px' },
  { key: 'is_anomaly', label: 'Anomaly', width: '64px' },
];

const COLORS = ['#e74c3c', '#f39c12', '#3498db', '#9b59b6', '#1abc9c', '#34495e', '#e67e22'];

function formatNum(v: unknown): string {
  if (v == null || v === '') return '';
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return String(v);
  return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

/** Fill scenario_display so the Scenario column is never empty: scenario_id || cluster_id || causal_reason_type */
function normalizeLabels(rows: AnomalyLabel[]): AnomalyLabel[] {
  return rows.map((r) => {
    const scenario =
      (r.scenario_id as string)?.trim() ||
      (r.cluster_id as string)?.trim() ||
      (r.causal_reason_type as string)?.trim() ||
      '—';
    return { ...r, scenario_display: scenario };
  });
}

export function FraudAnomalyView() {
  const [labels, setLabels] = useState<AnomalyLabel[]>([]);
  const [journalRows, setJournalRows] = useState<JournalEntryRow[]>([]);
  const [selectedLabel, setSelectedLabel] = useState<AnomalyLabel | null>(null);
  const [categoryFilter, setCategoryFilter] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([loadAnomalyLabels(), loadFraudLabels(), loadJournalEntriesCsv().catch(() => [])])
      .then(([anomaly, fraud, jeRows]) => {
        const combined: AnomalyLabel[] = [];
        if (anomaly?.length) combined.push(...anomaly);
        if (fraud?.length) {
          fraud.forEach((f) => {
            if (!combined.some((a) => (a.anomaly_id || a.document_id) === (f.anomaly_id || f.document_id)))
              combined.push(f);
          });
        }
        setLabels(combined);
        setJournalRows(jeRows ?? []);
        setError(null);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load labels');
        setLabels([]);
        setJournalRows([]);
      })
      .finally(() => setLoading(false));
  }, []);

  const displayLabels = useMemo(() => normalizeLabels(labels), [labels]);

  const categoryOptions = useMemo(() => {
    const set = new Set<string>();
    displayLabels.forEach((l) => {
      const cat = (l.anomaly_category ?? l.anomaly_type ?? 'Other') as string;
      if (cat) set.add(cat);
    });
    return ['', ...Array.from(set).sort()];
  }, [displayLabels]);

  const filteredLabels = useMemo(() => {
    if (!categoryFilter) return displayLabels;
    return displayLabels.filter(
      (l) => (l.anomaly_category ?? l.anomaly_type ?? 'Other') === categoryFilter
    );
  }, [displayLabels, categoryFilter]);

  const byCategory = useMemo(() => {
    const map: Record<string, number> = {};
    filteredLabels.forEach((l) => {
      const cat = (l.anomaly_category ?? l.anomaly_type ?? 'Other') as string;
      map[cat] = (map[cat] ?? 0) + 1;
    });
    return Object.entries(map).map(([name, count]) => ({ name, count }));
  }, [filteredLabels]);

  const byType = useMemo(() => {
    const map: Record<string, number> = {};
    filteredLabels.forEach((l) => {
      const t = (l.anomaly_type ?? 'Other') as string;
      map[t] = (map[t] ?? 0) + 1;
    });
    return Object.entries(map)
      .map(([name, value]) => ({ name, value }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 12);
  }, [filteredLabels]);

  const byScheme = useMemo(() => {
    const map: Record<string, number> = {};
    filteredLabels.forEach((l) => {
      const s = (l.scenario_display ?? '—') as string;
      map[s] = (map[s] ?? 0) + 1;
    });
    return Object.entries(map)
      .map(([name, value]) => ({ name, value }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 10);
  }, [filteredLabels]);

  const pieData = useMemo(
    () => byCategory.map((d, i) => ({ ...d, fill: COLORS[i % COLORS.length] })),
    [byCategory]
  );

  const concernedRows = useMemo(() => {
    if (!selectedLabel?.document_id) return [];
    return journalRows.filter((r) => r.document_id === selectedLabel.document_id);
  }, [selectedLabel, journalRows]);

  const concernedChartData = useMemo(() => {
    return concernedRows.map((r, i) => {
      const debit = typeof r.debit_amount === 'number' ? r.debit_amount : parseFloat(String(r.debit_amount || 0));
      const credit = typeof r.credit_amount === 'number' ? r.credit_amount : parseFloat(String(r.credit_amount || 0));
      return {
        line: `L${r.line_number ?? i + 1}`,
        debit: Number.isFinite(debit) ? debit : 0,
        credit: Number.isFinite(credit) ? credit : 0,
        gl_account: r.gl_account ?? '',
      };
    });
  }, [concernedRows]);

  const [expandedChart, setExpandedChart] = useState<'category' | 'type' | 'scheme' | null>(null);

  const ExpandIcon = () => (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3" />
    </svg>
  );

  if (loading) return <div className="fraud-anomaly-view loading">Loading fraud and anomaly labels…</div>;
  if (error && labels.length === 0) return <div className="fraud-anomaly-view error">Error: {error}</div>;

  return (
    <div className="fraud-anomaly-view">
      <h2>Fraud, Anomalies &amp; Schemes</h2>
      <p className="fraud-anomaly-desc">
        Labels from <code>labels/anomaly_labels</code> and <code>labels/fraud_labels</code>. Click a row to view the
        concerned transaction. Scenario shows scheme/cluster or causal reason when no scenario_id is set.
      </p>
      {labels.length === 0 ? (
        <p className="fraud-anomaly-empty">
          No anomaly or fraud labels in output. Enable anomaly injection in config to generate labels.
        </p>
      ) : (
        <>
          <div className="fraud-anomaly-filter">
            <label htmlFor="fraud-category-filter">Filter by category:</label>
            <select
              id="fraud-category-filter"
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value)}
            >
              <option value="">All categories</option>
              {categoryOptions
                .filter((c) => c !== '')
                .map((cat) => (
                  <option key={cat} value={cat}>
                    {cat}
                  </option>
                ))}
            </select>
            <span className="fraud-anomaly-filter-count">
              {filteredLabels.length} label{filteredLabels.length !== 1 ? 's' : ''}
            </span>
          </div>
          <div className="fraud-anomaly-charts">
            <div className="chart-box">
              <div className="chart-box-header">
                <h3>By Category</h3>
                <button
                  type="button"
                  className="chart-expand-btn"
                  onClick={() => setExpandedChart('category')}
                  title="View larger"
                  aria-label="View By Category chart larger"
                >
                  <ExpandIcon />
                </button>
              </div>
              <ResponsiveContainer width="100%" height={260}>
                <PieChart>
                  <Pie data={pieData} dataKey="count" nameKey="name" cx="50%" cy="50%" outerRadius={90} label={false}>
                    {pieData.map((entry) => (
                      <Cell key={entry.name} fill={entry.fill} />
                    ))}
                  </Pie>
                  <Tooltip />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            </div>
            <div className="chart-box">
              <div className="chart-box-header">
                <h3>By Type (top 12)</h3>
                <button
                  type="button"
                  className="chart-expand-btn"
                  onClick={() => setExpandedChart('type')}
                  title="View larger"
                  aria-label="View By Type chart larger"
                >
                  <ExpandIcon />
                </button>
              </div>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={byType} layout="vertical" margin={{ top: 5, right: 20, left: 120, bottom: 20 }}>
                  <XAxis type="number" tick={{ fontSize: 11 }} />
                  <YAxis dataKey="name" type="category" width={120} tick={{ fontSize: 11 }} interval={0} />
                  <Tooltip />
                  <Bar dataKey="value" fill="#4a7cff" name="Count" radius={[0, 4, 4, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </div>
            <div className="chart-box">
              <div className="chart-box-header">
                <h3>By Scenario / Scheme (top 10)</h3>
                <button
                  type="button"
                  className="chart-expand-btn"
                  onClick={() => setExpandedChart('scheme')}
                  title="View larger"
                  aria-label="View By Scenario chart larger"
                >
                  <ExpandIcon />
                </button>
              </div>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={byScheme} layout="vertical" margin={{ top: 5, right: 20, left: 120, bottom: 20 }}>
                  <XAxis type="number" tick={{ fontSize: 11 }} />
                  <YAxis dataKey="name" type="category" width={120} tick={{ fontSize: 11 }} interval={0} />
                  <Tooltip />
                  <Bar dataKey="value" fill="#9b59b6" name="Count" radius={[0, 4, 4, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </div>
          {expandedChart && (
            <div
              className="chart-modal-overlay"
              role="dialog"
              aria-modal="true"
              aria-label={`${expandedChart} chart expanded`}
              onClick={() => setExpandedChart(null)}
            >
              <div className="chart-modal" onClick={(e) => e.stopPropagation()}>
                <div className="chart-modal-header">
                  <h3>
                    {expandedChart === 'category' && 'By Category'}
                    {expandedChart === 'type' && 'By Type (top 12)'}
                    {expandedChart === 'scheme' && 'By Scenario / Scheme (top 10)'}
                  </h3>
                  <button
                    type="button"
                    className="chart-modal-close"
                    onClick={() => setExpandedChart(null)}
                    aria-label="Close"
                  >
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                  </button>
                </div>
                <div className="chart-modal-body">
                  {expandedChart === 'category' && (
                    <ResponsiveContainer width="100%" height={420}>
                      <PieChart>
                        <Pie data={pieData} dataKey="count" nameKey="name" cx="50%" cy="50%" outerRadius={140} label={false}>
                          {pieData.map((entry) => (
                            <Cell key={entry.name} fill={entry.fill} />
                          ))}
                        </Pie>
                        <Tooltip />
                        <Legend />
                      </PieChart>
                    </ResponsiveContainer>
                  )}
                  {expandedChart === 'type' && (
                    <ResponsiveContainer width="100%" height={420}>
                      <BarChart
                        data={byType}
                        layout="vertical"
                        margin={{ top: 8, right: 24, left: 140, bottom: 24 }}
                      >
                        <XAxis type="number" tick={{ fontSize: 12 }} />
                        <YAxis dataKey="name" type="category" width={140} tick={{ fontSize: 12 }} interval={0} />
                        <Tooltip />
                        <Bar dataKey="value" fill="#4a7cff" name="Count" radius={[0, 4, 4, 0]} />
                      </BarChart>
                    </ResponsiveContainer>
                  )}
                  {expandedChart === 'scheme' && (
                    <ResponsiveContainer width="100%" height={420}>
                      <BarChart
                        data={byScheme}
                        layout="vertical"
                        margin={{ top: 8, right: 24, left: 140, bottom: 24 }}
                      >
                        <XAxis type="number" tick={{ fontSize: 12 }} />
                        <YAxis dataKey="name" type="category" width={140} tick={{ fontSize: 12 }} interval={0} />
                        <Tooltip />
                        <Bar dataKey="value" fill="#9b59b6" name="Count" radius={[0, 4, 4, 0]} />
                      </BarChart>
                    </ResponsiveContainer>
                  )}
                </div>
              </div>
            </div>
          )}
          <h3>Label table (click a row to view concerned transaction)</h3>
          <DataTable
            data={filteredLabels as Record<string, unknown>[]}
            columns={TABLE_COLUMNS}
            keyField="anomaly_id"
            pageSize={50}
            maxHeight="40vh"
            onRowClick={(row) => setSelectedLabel(row as AnomalyLabel)}
            selectedRowKey={selectedLabel?.anomaly_id ?? null}
          />
          {selectedLabel && (
            <div className="fraud-anomaly-concerned">
              <h3>Concerned transaction: {selectedLabel.document_id}</h3>
              <p className="fraud-anomaly-concerned-desc">
                Journal entry lines for document <code>{selectedLabel.document_id}</code> (anomaly{' '}
                {selectedLabel.anomaly_id}, {selectedLabel.anomaly_type ?? selectedLabel.anomaly_category}).
              </p>
              {concernedRows.length === 0 ? (
                <p className="fraud-anomaly-concerned-empty">
                  No journal lines found for this document in <code>journal_entries.csv</code>. The document may be from
                  another run or format.
                </p>
              ) : (
                <>
                  <div className="fraud-anomaly-concerned-chart">
                    <ResponsiveContainer width="100%" height={220}>
                      <BarChart
                        data={concernedChartData}
                        margin={{ top: 5, right: 20, left: 5, bottom: 30 }}
                        barCategoryGap="20%"
                      >
                        <XAxis dataKey="line" tick={{ fontSize: 11 }} />
                        <YAxis tickFormatter={(v) => (v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v.toLocaleString())} />
                        <Tooltip formatter={(v: number | undefined) => formatNum(v)} />
                        <Bar dataKey="debit" name="Debit" fill="#2ecc71" radius={[4, 4, 0, 0]} />
                        <Bar dataKey="credit" name="Credit" fill="#e74c3c" radius={[4, 4, 0, 0]} />
                        <Legend />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                  <DataTable
                    data={concernedRows.map((r, i) => ({ ...r, _rowKey: `${r.document_id}-${r.line_number ?? i}` })) as unknown as Record<string, unknown>[]}
                    columns={CONCERNED_FULL_COLUMNS}
                    keyField="_rowKey"
                    pageSize={20}
                    maxHeight="40vh"
                  />
                </>
              )}
            </div>
          )}
        </>
      )}
    </div>
  );
}
