import { useEffect, useState, useMemo, useCallback } from 'react';
import {
  loadSubledgerAr,
  loadSubledgerAp,
  loadSubledgerFa,
  loadSubledgerInventory,
  loadSubledgerReconciliation,
  loadChartOfAccounts,
} from '../api/data';
import { DataTable } from '../components/DataTable';
import './SubledgerView.css';

type SubledgerTab = 'ar' | 'ap' | 'fa' | 'inventory' | 'reconciliation';
type ViewMode = 'document' | 'entries';
type SubledgerLang = 'fr' | 'en';

const TABS_EN: { id: SubledgerTab; label: string }[] = [
  { id: 'ar', label: 'AR (Receivables)' },
  { id: 'ap', label: 'AP (Payables)' },
  { id: 'fa', label: 'Fixed Assets' },
  { id: 'inventory', label: 'Inventory' },
  { id: 'reconciliation', label: 'Reconciliation' },
];
const TABS_FR: { id: SubledgerTab; label: string }[] = [
  { id: 'ar', label: 'Créances (AR)' },
  { id: 'ap', label: 'Dettes (AP)' },
  { id: 'fa', label: 'Immobilisations' },
  { id: 'inventory', label: 'Stocks' },
  { id: 'reconciliation', label: 'Réconciliation' },
];

const identityStr = (x: string) => x;

function formatNum(v: unknown): string {
  if (v == null || v === '') return '';
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  if (Number.isNaN(n)) return String(v);
  return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

/** Flatten AR invoice for table: top-level scalars + net_amount, line count, account (FEC-style from lines). Includes posting_date. */
function flattenAr(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  return rows.map((r) => {
    const net = r.net_amount as Record<string, unknown> | undefined;
    const lines = r.lines as Record<string, unknown>[] | undefined;
    const accounts = new Set<string>();
    if (Array.isArray(lines)) {
      for (const line of lines) {
        const rev = (line.revenue_account ?? line.gl_account ?? '').toString().trim();
        if (rev) accounts.add(rev);
      }
    }
    return {
      ...r,
      _rowKey: r.invoice_number ?? r.document_id ?? Math.random(),
      net_amount_display: net && typeof net.document_amount !== 'undefined' ? net.document_amount : (net?.local_amount ?? ''),
      line_count: Array.isArray(lines) ? lines.length : 0,
      account_display: Array.from(accounts).join(', ') || '',
      posting_date: r.posting_date ?? r.invoice_date ?? '',
    };
  });
}

/** Flatten AP invoice for table: include account (FEC-style from lines). Includes posting_date. */
function flattenAp(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  return rows.map((r) => {
    const net = r.net_amount as Record<string, unknown> | undefined;
    const lines = r.lines as Record<string, unknown>[] | undefined;
    const accounts = new Set<string>();
    if (Array.isArray(lines)) {
      for (const line of lines) {
        const gl = (line.gl_account ?? line.revenue_account ?? '').toString().trim();
        if (gl) accounts.add(gl);
      }
    }
    return {
      ...r,
      _rowKey: r.invoice_number ?? r.vendor_invoice_number ?? Math.random(),
      net_amount_display: net && typeof net.document_amount !== 'undefined' ? net.document_amount : (net?.local_amount ?? ''),
      line_count: Array.isArray(lines) ? lines.length : 0,
      account_display: Array.from(accounts).join(', ') || '',
      posting_date: r.posting_date ?? r.invoice_date ?? '',
    };
  });
}

/** One row per AR line (journal entry level). All dates on each row. */
function flattenArToEntries(
  rows: Record<string, unknown>[],
  normalizeAccount: (raw: string) => string
): Record<string, unknown>[] {
  const out: Record<string, unknown>[] = [];
  for (const r of rows) {
    const lines = (r.lines as Record<string, unknown>[] | undefined) ?? [];
    const postingDate = (r.posting_date ?? r.invoice_date ?? '') as string;
    const invoiceDate = (r.invoice_date ?? '') as string;
    const dueDate = (r.due_date ?? '') as string;
    for (const line of lines) {
      const rev = (line.revenue_account ?? line.gl_account ?? '').toString().trim();
      const net = line.net_amount ?? '';
      out.push({
        _rowKey: `${r.invoice_number ?? r.document_id}-${line.line_number ?? out.length}`,
        invoice_number: r.invoice_number ?? r.document_id,
        company_code: r.company_code,
        posting_date: postingDate,
        invoice_date: invoiceDate,
        due_date: dueDate,
        customer_id: r.customer_id,
        customer_name: r.customer_name,
        line_number: line.line_number,
        description: line.description,
        account_display: rev ? normalizeAccount(rev) : '',
        quantity: line.quantity,
        unit: line.unit,
        unit_price: line.unit_price,
        net_amount: net,
        tax_amount: line.tax_amount ?? '',
        gross_amount: line.gross_amount ?? '',
        debit_amount: line.debit_amount ?? 0,
        credit_amount: line.credit_amount ?? line.net_amount ?? 0,
        reference_piece: r.invoice_number ?? r.document_id,
        date_piece: invoiceDate,
        libelle_ecriture: line.description ?? line.line_text ?? '',
        lettrage: r.lettrage ?? line.lettrage ?? '',
        lettrage_date: r.lettrage_date ?? line.lettrage_date ?? '',
        journal_code: r.journal_code ?? 'VT',
      });
    }
  }
  return out;
}

/** One row per AP line (journal entry level). All dates on each row. */
function flattenApToEntries(
  rows: Record<string, unknown>[],
  normalizeAccount: (raw: string) => string
): Record<string, unknown>[] {
  const out: Record<string, unknown>[] = [];
  for (const r of rows) {
    const lines = (r.lines as Record<string, unknown>[] | undefined) ?? [];
    const postingDate = (r.posting_date ?? r.invoice_date ?? '') as string;
    const invoiceDate = (r.invoice_date ?? '') as string;
    const dueDate = (r.due_date ?? '') as string;
    for (const line of lines) {
      const gl = (line.gl_account ?? line.revenue_account ?? '').toString().trim();
      const net = line.net_amount ?? '';
      out.push({
        _rowKey: `${r.invoice_number ?? r.vendor_invoice_number}-${line.line_number ?? out.length}`,
        invoice_number: r.invoice_number,
        vendor_invoice_number: r.vendor_invoice_number,
        company_code: r.company_code,
        posting_date: postingDate,
        invoice_date: invoiceDate,
        due_date: dueDate,
        vendor_id: r.vendor_id,
        vendor_name: r.vendor_name,
        line_number: line.line_number,
        description: line.description,
        account_display: gl ? normalizeAccount(gl) : '',
        quantity: line.quantity,
        unit: line.unit,
        unit_price: line.unit_price,
        net_amount: net,
        tax_amount: line.tax_amount ?? '',
        gross_amount: line.gross_amount ?? '',
        debit_amount: line.debit_amount ?? line.net_amount ?? 0,
        credit_amount: line.credit_amount ?? 0,
        reference_piece: r.vendor_invoice_number ?? r.invoice_number,
        date_piece: invoiceDate,
        libelle_ecriture: line.description ?? line.line_text ?? '',
        lettrage: r.lettrage ?? line.lettrage ?? '',
        lettrage_date: r.lettrage_date ?? line.lettrage_date ?? '',
        journal_code: r.journal_code ?? 'AC',
      });
    }
  }
  return out;
}

/** Flatten FA record: include account (acquisition account from account_determination) */
function flattenFa(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  return rows.map((r) => {
    const det = r.account_determination as Record<string, unknown> | undefined;
    const acq = det?.acquisition_account ?? r.gl_account ?? '';
    return {
      ...r,
      _rowKey: r.asset_number ?? Math.random(),
      account_display: acq != null ? String(acq).trim() : '',
    };
  });
}

/** Flatten inventory: pull valuation method and unit cost */
function flattenInventory(rows: Record<string, unknown>[]): Record<string, unknown>[] {
  return rows.map((r) => {
    const val = r.valuation as Record<string, unknown> | undefined;
    return {
      ...r,
      _rowKey: r.material_id ?? Math.random(),
      valuation_method: val?.method ?? '',
      unit_cost: val?.unit_cost ?? val?.standard_cost ?? '',
      total_value: val?.total_value ?? '',
    };
  });
}

type Col = { key: string; label: string; width: string; format?: (v: unknown) => string };

function arColumns(lang: SubledgerLang, showFec: boolean): Col[] {
  const L = lang === 'fr';
  const base: Col[] = [
    { key: 'invoice_number', label: L ? 'N° Facture' : 'Invoice', width: '140px' },
    { key: 'company_code', label: L ? 'Société' : 'Company', width: '80px' },
    { key: 'account_display', label: L ? 'Compte' : 'Account', width: '90px' },
    { key: 'customer_id', label: L ? 'Client (ID)' : 'Customer ID', width: '100px' },
    { key: 'customer_name', label: L ? 'Client' : 'Customer', width: '160px' },
    { key: 'posting_date', label: L ? 'Date comptab.' : 'Posting Date', width: '100px' },
    { key: 'invoice_date', label: L ? 'Date facture' : 'Invoice Date', width: '100px' },
    { key: 'due_date', label: L ? 'Échéance' : 'Due Date', width: '100px' },
    { key: 'invoice_type', label: L ? 'Type' : 'Type', width: '90px' },
    { key: 'status', label: L ? 'Statut' : 'Status', width: '80px' },
    { key: 'net_amount_display', label: L ? 'Montant net' : 'Net Amount', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'amount_remaining', label: L ? 'Restant dû' : 'Remaining', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'line_count', label: L ? 'Lignes' : 'Lines', width: '60px' },
  ];
  return base;
}

function apColumns(lang: SubledgerLang, _showFec: boolean): Col[] {
  const L = lang === 'fr';
  return [
    { key: 'invoice_number', label: L ? 'N° Facture' : 'Invoice', width: '120px' },
    { key: 'vendor_invoice_number', label: L ? 'Réf. fournisseur' : 'Vendor Ref', width: '120px' },
    { key: 'company_code', label: L ? 'Société' : 'Company', width: '80px' },
    { key: 'account_display', label: L ? 'Compte' : 'Account', width: '90px' },
    { key: 'vendor_id', label: L ? 'Fournisseur (ID)' : 'Vendor ID', width: '100px' },
    { key: 'vendor_name', label: L ? 'Fournisseur' : 'Vendor', width: '160px' },
    { key: 'posting_date', label: L ? 'Date comptab.' : 'Posting Date', width: '100px' },
    { key: 'invoice_date', label: L ? 'Date facture' : 'Invoice Date', width: '100px' },
    { key: 'due_date', label: L ? 'Échéance' : 'Due Date', width: '100px' },
    { key: 'invoice_type', label: L ? 'Type' : 'Type', width: '90px' },
    { key: 'status', label: L ? 'Statut' : 'Status', width: '80px' },
    { key: 'net_amount_display', label: L ? 'Montant net' : 'Net Amount', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'amount_remaining', label: L ? 'Restant dû' : 'Remaining', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'line_count', label: L ? 'Lignes' : 'Lines', width: '60px' },
  ];
}

function arEntriesColumns(lang: SubledgerLang, showFec: boolean): Col[] {
  const L = lang === 'fr';
  if (showFec) {
    return [
      { key: 'journal_code', label: 'Code journal', width: '80px' },
      { key: 'posting_date', label: 'Date de comptabilisation', width: '110px' },
      { key: 'invoice_number', label: "Réf. pièce justificative", width: '120px' },
      { key: 'date_piece', label: "Date d'émission pièce", width: '110px' },
      { key: 'due_date', label: 'Date échéance', width: '100px' },
      { key: 'account_display', label: 'Numéro de compte', width: '100px' },
      { key: 'debit_amount', label: 'Débit', width: '100px', format: (v: unknown) => formatNum(v) },
      { key: 'credit_amount', label: 'Crédit', width: '100px', format: (v: unknown) => formatNum(v) },
      { key: 'lettrage', label: 'Lettrage', width: '80px' },
      { key: 'lettrage_date', label: 'Date de lettrage', width: '100px' },
      { key: 'libelle_ecriture', label: "Libellé de l'écriture comptable", width: '200px' },
      { key: 'customer_name', label: 'Client', width: '140px' },
      { key: 'net_amount', label: 'Montant net', width: '110px', format: (v: unknown) => formatNum(v) },
    ];
  }
  return [
    { key: 'invoice_number', label: L ? 'N° Facture' : 'Invoice', width: '120px' },
    { key: 'company_code', label: L ? 'Société' : 'Company', width: '80px' },
    { key: 'posting_date', label: L ? 'Date comptab.' : 'Posting Date', width: '100px' },
    { key: 'invoice_date', label: L ? 'Date facture' : 'Invoice Date', width: '100px' },
    { key: 'due_date', label: L ? 'Échéance' : 'Due Date', width: '100px' },
    { key: 'customer_id', label: L ? 'Client (ID)' : 'Customer ID', width: '100px' },
    { key: 'customer_name', label: L ? 'Client' : 'Customer', width: '140px' },
    { key: 'line_number', label: L ? 'N° ligne' : 'Line', width: '60px' },
    { key: 'account_display', label: L ? 'Compte' : 'Account', width: '90px' },
    { key: 'description', label: L ? 'Libellé' : 'Description', width: '180px' },
    { key: 'net_amount', label: L ? 'Montant net' : 'Net Amount', width: '110px', format: (v: unknown) => formatNum(v) },
  ];
}

function apEntriesColumns(lang: SubledgerLang, showFec: boolean): Col[] {
  const L = lang === 'fr';
  if (showFec) {
    return [
      { key: 'journal_code', label: 'Code journal', width: '80px' },
      { key: 'posting_date', label: 'Date de comptabilisation', width: '110px' },
      { key: 'vendor_invoice_number', label: "Réf. pièce justificative", width: '120px' },
      { key: 'date_piece', label: "Date d'émission pièce", width: '110px' },
      { key: 'due_date', label: 'Date échéance', width: '100px' },
      { key: 'account_display', label: 'Numéro de compte', width: '100px' },
      { key: 'debit_amount', label: 'Débit', width: '100px', format: (v: unknown) => formatNum(v) },
      { key: 'credit_amount', label: 'Crédit', width: '100px', format: (v: unknown) => formatNum(v) },
      { key: 'lettrage', label: 'Lettrage', width: '80px' },
      { key: 'lettrage_date', label: 'Date de lettrage', width: '100px' },
      { key: 'libelle_ecriture', label: "Libellé de l'écriture comptable", width: '200px' },
      { key: 'vendor_name', label: 'Fournisseur', width: '140px' },
      { key: 'net_amount', label: 'Montant net', width: '110px', format: (v: unknown) => formatNum(v) },
    ];
  }
  return [
    { key: 'invoice_number', label: L ? 'N° Facture' : 'Invoice', width: '120px' },
    { key: 'vendor_invoice_number', label: L ? 'Réf. fournisseur' : 'Vendor Ref', width: '120px' },
    { key: 'company_code', label: L ? 'Société' : 'Company', width: '80px' },
    { key: 'posting_date', label: L ? 'Date comptab.' : 'Posting Date', width: '100px' },
    { key: 'invoice_date', label: L ? 'Date facture' : 'Invoice Date', width: '100px' },
    { key: 'due_date', label: L ? 'Échéance' : 'Due Date', width: '100px' },
    { key: 'vendor_id', label: L ? 'Fournisseur (ID)' : 'Vendor ID', width: '100px' },
    { key: 'vendor_name', label: L ? 'Fournisseur' : 'Vendor', width: '140px' },
    { key: 'line_number', label: L ? 'N° ligne' : 'Line', width: '60px' },
    { key: 'account_display', label: L ? 'Compte' : 'Account', width: '90px' },
    { key: 'description', label: L ? 'Libellé' : 'Description', width: '180px' },
    { key: 'net_amount', label: L ? 'Montant net' : 'Net Amount', width: '110px', format: (v: unknown) => formatNum(v) },
  ];
}

function faColumns(lang: SubledgerLang): Col[] {
  const L = lang === 'fr';
  return [
    { key: 'asset_number', label: L ? 'Immobilisation' : 'Asset', width: '120px' },
    { key: 'company_code', label: L ? 'Société' : 'Company', width: '80px' },
    { key: 'account_display', label: L ? 'Compte' : 'Account', width: '90px' },
    { key: 'asset_class', label: L ? 'Classe' : 'Class', width: '100px' },
    { key: 'description', label: L ? 'Libellé' : 'Description', width: '180px' },
    { key: 'status', label: L ? 'Statut' : 'Status', width: '80px' },
    { key: 'acquisition_date', label: L ? 'Date acquisition' : 'Acquisition', width: '100px' },
    { key: 'capitalization_date', label: L ? 'Date mise en service' : 'Capitalized', width: '110px' },
    { key: 'acquisition_cost', label: L ? 'Coût' : 'Cost', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'accumulated_depreciation', label: L ? 'Amort. cumulé' : 'Accum. Depr.', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'net_book_value', label: L ? 'VNC' : 'NBV', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'currency', label: L ? 'Devise' : 'CCY', width: '50px' },
  ];
}

function inventoryColumns(lang: SubledgerLang): Col[] {
  const L = lang === 'fr';
  return [
    { key: 'material_id', label: L ? 'Article' : 'Material', width: '120px' },
    { key: 'description', label: L ? 'Libellé' : 'Description', width: '180px' },
    { key: 'plant', label: L ? 'Centre' : 'Plant', width: '90px' },
    { key: 'storage_location', label: L ? 'Emplacement' : 'Storage', width: '90px' },
    { key: 'company_code', label: L ? 'Société' : 'Company', width: '80px' },
    { key: 'quantity_on_hand', label: L ? 'En stock' : 'On Hand', width: '90px' },
    { key: 'quantity_available', label: L ? 'Disponible' : 'Available', width: '90px' },
    { key: 'unit', label: L ? 'Unité' : 'Unit', width: '50px' },
    { key: 'valuation_method', label: L ? 'Valorisation' : 'Valuation', width: '100px' },
    { key: 'unit_cost', label: L ? 'Coût unit.' : 'Unit Cost', width: '100px', format: (v: unknown) => formatNum(v) },
    { key: 'total_value', label: L ? 'Valeur totale' : 'Total Value', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'status', label: L ? 'Statut' : 'Status', width: '80px' },
  ];
}

/** Columns for AR/AP invoice line items in the accordion. */
function invoiceLineColumns(lang: SubledgerLang): Col[] {
  const L = lang === 'fr';
  return [
    { key: 'line_number', label: L ? 'N°' : '#', width: '44px' },
    { key: 'description', label: L ? 'Libellé' : 'Description', width: '200px' },
    { key: 'quantity', label: L ? 'Qté' : 'Qty', width: '70px' },
    { key: 'unit', label: L ? 'Unité' : 'Unit', width: '50px' },
    { key: 'unit_price', label: L ? 'Prix unit.' : 'Unit Price', width: '100px', format: (v: unknown) => formatNum(v) },
    { key: 'net_amount', label: L ? 'Montant net' : 'Net Amount', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'tax_amount', label: L ? 'Taxe' : 'Tax', width: '90px', format: (v: unknown) => formatNum(v) },
    { key: 'gross_amount', label: L ? 'Brut' : 'Gross', width: '110px', format: (v: unknown) => formatNum(v) },
    { key: 'gl_account_display', label: L ? 'Compte' : 'GL Account', width: '90px' },
  ];
}

function formatUnknown(v: unknown): string {
  if (v == null) return '';
  if (typeof v === 'object') return JSON.stringify(v).slice(0, 120);
  return String(v);
}

/** Return true if every non-empty account code in accountDisplay (comma-separated) exists in COA or is a prefix of a COA account. */
function accountDisplayInCoa(accountDisplay: string, coaAccountSet: Set<string>): boolean {
  if (coaAccountSet.size === 0) return true;
  const parts = (accountDisplay ?? '').split(',').map((p) => p.trim()).filter(Boolean);
  if (parts.length === 0) return true;
  return parts.every(
    (part) =>
      coaAccountSet.has(part) ||
      [...coaAccountSet].some((c) => c === part || c.startsWith(part) || part.startsWith(c))
  );
}

/** Normalize account display to COA account numbers: replace each part with first matching COA account (exact or prefix). */
function normalizeAccountDisplayToCoa(
  accountDisplay: string,
  coaAccountList: string[]
): string {
  if (coaAccountList.length === 0) return accountDisplay;
  const parts = (accountDisplay ?? '').split(',').map((p) => p.trim()).filter(Boolean);
  if (parts.length === 0) return accountDisplay;
  const normalized = parts.map((part) => {
    if (coaAccountList.includes(part)) return part;
    const match = coaAccountList.find((c) => c.startsWith(part) || part.startsWith(c));
    return match ?? part;
  });
  return normalized.join(', ');
}

export function SubledgerView() {
  const [activeTab, setActiveTab] = useState<SubledgerTab>('ar');
  const [viewMode, setViewMode] = useState<ViewMode>('entries');
  const [showFecColumns, setShowFecColumns] = useState(false);
  const [lang, setLang] = useState<SubledgerLang>('fr');
  const [selectedRow, setSelectedRow] = useState<Record<string, unknown> | null>(null);
  const [selectedEntryKey, setSelectedEntryKey] = useState<string | null>(null);
  const [ar, setAr] = useState<Record<string, unknown>[]>([]);
  const [ap, setAp] = useState<Record<string, unknown>[]>([]);
  const [fa, setFa] = useState<Record<string, unknown>[]>([]);
  const [inventory, setInventory] = useState<Record<string, unknown>[]>([]);
  const [reconciliation, setReconciliation] = useState<Record<string, unknown>[]>([]);
  const [coaAccountSet, setCoaAccountSet] = useState<Set<string>>(new Set());
  const [coaAccountList, setCoaAccountList] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const normalizeAccount = useCallback(
    (raw: string) =>
      coaAccountList.length > 0 ? normalizeAccountDisplayToCoa(raw, coaAccountList) : raw,
    [coaAccountList]
  );

  const selectedRowKey =
    viewMode === 'entries' && (activeTab === 'ar' || activeTab === 'ap')
      ? selectedEntryKey
      : selectedRow
        ? String(selectedRow._rowKey ?? selectedRow.invoice_number ?? selectedRow.vendor_invoice_number ?? selectedRow.asset_number ?? selectedRow.material_id ?? '')
        : null;
  const hasLines = (activeTab === 'ar' || activeTab === 'ap') && selectedRow && Array.isArray(selectedRow.lines);
  const lineRows = useMemo(() => {
    if (!hasLines || !selectedRow?.lines) return [];
    const lines = selectedRow.lines as Record<string, unknown>[];
    const rawAccount = (line: Record<string, unknown>) =>
      String(line.revenue_account ?? line.gl_account ?? '').trim();
    return lines.map((line) => {
      const raw = rawAccount(line);
      const display =
        coaAccountList.length > 0
          ? normalizeAccountDisplayToCoa(raw, coaAccountList)
          : raw;
      return { ...line, gl_account_display: display || raw };
    });
  }, [hasLines, selectedRow?.lines, coaAccountList]);

  useEffect(() => {
    Promise.all([
      loadSubledgerAr(),
      loadSubledgerAp(),
      loadSubledgerFa(),
      loadSubledgerInventory(),
      loadSubledgerReconciliation(),
      loadChartOfAccounts(),
    ])
      .then(([arData, apData, faData, invData, recData, coa]) => {
        setAr(arData);
        setAp(apData);
        setFa(faData);
        setInventory(invData);
        setReconciliation(recData);
        const list = Array.from(coa.keys());
        setCoaAccountSet(new Set(list));
        setCoaAccountList(list);
        setError(null);
      })
      .catch((e) => {
        setError(e instanceof Error ? e.message : 'Failed to load subledger data');
        setAr([]);
        setAp([]);
        setFa([]);
        setInventory([]);
        setReconciliation([]);
        setCoaAccountSet(new Set());
        setCoaAccountList([]);
      })
      .finally(() => setLoading(false));
  }, []);

  const handleTabChange = (tab: SubledgerTab) => {
    setActiveTab(tab);
    setSelectedRow(null);
    setSelectedEntryKey(null);
  };

  const handleRowClick = (row: Record<string, unknown>) => {
    let doc: Record<string, unknown> | null = row;
    const entryKey = String(row._rowKey ?? row.invoice_number ?? row.vendor_invoice_number ?? '');
    if (viewMode === 'entries' && (activeTab === 'ar' || activeTab === 'ap')) {
      const inv = String(row.invoice_number ?? row.vendor_invoice_number ?? '');
      if (inv) {
        const list = activeTab === 'ar' ? ar : ap;
        doc = list.find((d) => String(d.invoice_number ?? d.vendor_invoice_number ?? d.document_id ?? '') === inv) ?? row;
      }
      setSelectedEntryKey((prev) => (prev === entryKey ? null : entryKey));
    } else {
      setSelectedEntryKey(null);
    }
    const rowKey = String(doc?._rowKey ?? doc?.invoice_number ?? doc?.vendor_invoice_number ?? doc?.asset_number ?? doc?.material_id ?? '');
    setSelectedRow((prev) => {
      if (!prev) return doc;
      const prevKey = String(prev._rowKey ?? prev.invoice_number ?? prev.vendor_invoice_number ?? prev.asset_number ?? prev.material_id ?? '');
      return prevKey === rowKey ? null : doc;
    });
  };

  const displayData = useMemo(() => {
    const filterByCoa = (rows: Record<string, unknown>[]) =>
      coaAccountSet.size === 0
        ? rows
        : rows.filter((row) => accountDisplayInCoa(String(row.account_display ?? ''), coaAccountSet));

    const normalizeCoa = (rows: Record<string, unknown>[]) =>
      coaAccountList.length === 0
        ? rows
        : rows.map((row) => ({
            ...row,
            account_display: normalizeAccountDisplayToCoa(String(row.account_display ?? ''), coaAccountList),
          }));

    switch (activeTab) {
      case 'ar':
        if (viewMode === 'entries') {
          const entries = flattenArToEntries(ar, normalizeAccount);
          return coaAccountSet.size === 0
            ? entries
            : entries.filter((row) => accountDisplayInCoa(String(row.account_display ?? ''), coaAccountSet));
        }
        return normalizeCoa(filterByCoa(flattenAr(ar)));
      case 'ap':
        if (viewMode === 'entries') {
          const entries = flattenApToEntries(ap, normalizeAccount);
          return coaAccountSet.size === 0
            ? entries
            : entries.filter((row) => accountDisplayInCoa(String(row.account_display ?? ''), coaAccountSet));
        }
        return normalizeCoa(filterByCoa(flattenAp(ap)));
      case 'fa':
        return normalizeCoa(filterByCoa(flattenFa(fa)));
      case 'inventory':
        return flattenInventory(inventory);
      case 'reconciliation':
        return reconciliation.map((r, i) => ({ ...r, _rowKey: `rec-${i}` }));
      default:
        return [];
    }
  }, [activeTab, viewMode, ar, ap, fa, inventory, reconciliation, coaAccountSet, coaAccountList, normalizeAccount]);

  const columns = useMemo(() => {
    switch (activeTab) {
      case 'ar':
        return viewMode === 'entries' ? arEntriesColumns(lang, showFecColumns) : arColumns(lang, showFecColumns);
      case 'ap':
        return viewMode === 'entries' ? apEntriesColumns(lang, showFecColumns) : apColumns(lang, showFecColumns);
      case 'fa':
        return faColumns(lang);
      case 'inventory':
        return inventoryColumns(lang);
      case 'reconciliation':
        return reconciliation.length > 0 && typeof reconciliation[0] === 'object' && reconciliation[0] !== null
          ? Object.keys(reconciliation[0] as Record<string, unknown>)
              .filter((k) => !k.startsWith('_'))
              .slice(0, 14)
              .map((k) => ({
                key: k,
                label: k.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()),
                width: '120px',
                format: formatUnknown,
              }))
          : [];
      default:
        return [];
    }
  }, [activeTab, viewMode, lang, showFecColumns, reconciliation]);

  const keyField = useMemo(() => {
    switch (activeTab) {
      case 'ar':
      case 'ap':
        return '_rowKey';
      case 'fa':
        return 'asset_number';
      case 'inventory':
        return 'material_id';
      case 'reconciliation':
        return '_rowKey';
      default:
        return '_rowKey';
    }
  }, [activeTab, columns]);

  if (loading) {
    return <div className="subledger-view loading">Loading subledger data…</div>;
  }
  if (error && ar.length === 0 && ap.length === 0 && fa.length === 0 && inventory.length === 0 && reconciliation.length === 0) {
    return <div className="subledger-view error">Error: {error}</div>;
  }

  const counts = { ar: ar.length, ap: ap.length, fa: fa.length, inventory: inventory.length, reconciliation: reconciliation.length };
  const TABS = lang === 'fr' ? TABS_FR : TABS_EN;
  const showViewModeToggle = activeTab === 'ar' || activeTab === 'ap';

  return (
    <div className="subledger-view">
      <h2>{lang === 'fr' ? 'Sous-journaux et clôture' : 'Subledger & Period-Close'}</h2>
      <p className="subledger-desc">
        {lang === 'fr'
          ? 'Créances (factures clients, encaissements, échéances), dettes (factures fournisseurs, paiements), immobilisations (capitalisation, amortissements, cessions), stocks (valorisation FIFO/LIFO/moyenne pondérée). Réconciliation : balance/subledger_reconciliation.json.'
          : 'AR (customer invoices, receipts, aging), AP (vendor invoices, payments, aging), Fixed Assets (capitalization, depreciation, disposals), and Inventory (positions, valuation FIFO/LIFO/weighted average). Reconciliation from balance/subledger_reconciliation.json.'}
        {' '}
        <code>balance/subledger_reconciliation.json</code>
      </p>
      <div className="subledger-toolbar">
        <div className="subledger-tabs">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              className={activeTab === tab.id ? 'active' : ''}
              onClick={() => handleTabChange(tab.id)}
            >
              {tab.label} ({tab.id === 'ar' || tab.id === 'ap' ? (viewMode === 'entries' ? (tab.id === 'ar' ? flattenArToEntries(ar, identityStr).length : flattenApToEntries(ap, identityStr).length) : counts[tab.id]) : counts[tab.id]})
            </button>
          ))}
        </div>
        <div className="subledger-options">
          <label className="subledger-lang-toggle">
            <span className="subledger-option-label">{lang === 'fr' ? 'Langue' : 'Language'}:</span>
            <select value={lang} onChange={(e) => setLang(e.target.value as SubledgerLang)} aria-label={lang === 'fr' ? 'Langue' : 'Language'}>
              <option value="fr">Français</option>
              <option value="en">English</option>
            </select>
          </label>
          {showViewModeToggle && (
            <label className="subledger-view-mode">
              <span className="subledger-option-label">{lang === 'fr' ? 'Affichage' : 'View'}:</span>
              <select value={viewMode} onChange={(e) => setViewMode(e.target.value as ViewMode)} aria-label={lang === 'fr' ? 'Mode affichage' : 'View mode'}>
                <option value="entries">{lang === 'fr' ? 'Par écriture (toutes les lignes)' : 'By entry (all lines)'}</option>
                <option value="document">{lang === 'fr' ? 'Par document' : 'By document'}</option>
              </select>
            </label>
          )}
          {(activeTab === 'ar' || activeTab === 'ap') && (
            <label className="subledger-fec-toggle">
              <input
                type="checkbox"
                checked={showFecColumns}
                onChange={(e) => setShowFecColumns(e.target.checked)}
                aria-label={lang === 'fr' ? 'Colonnes FEC' : 'FEC columns'}
              />
              <span>{lang === 'fr' ? 'Colonnes FEC' : 'FEC columns'}</span>
            </label>
          )}
        </div>
      </div>
      {displayData.length === 0 ? (
        <p className="subledger-empty">
          {lang === 'fr'
            ? `Aucune donnée ${activeTab === 'ar' ? 'créances' : activeTab === 'ap' ? 'dettes' : activeTab === 'fa' ? 'immobilisations' : activeTab === 'inventory' ? 'stocks' : 'réconciliation'}. Activer la génération des sous-journaux dans la config.`
            : `No ${activeTab === 'ar' ? 'AR' : activeTab === 'ap' ? 'AP' : activeTab === 'fa' ? 'fixed asset' : activeTab === 'inventory' ? 'inventory' : 'reconciliation'} data in output. Enable subledger generation in config.`}
        </p>
      ) : (
        <>
          <p className="subledger-click-hint">
            {lang === 'fr' ? 'Cliquer sur une ligne pour afficher le détail des écritures (AR/AP).' : 'Click a row to expand and view line items (AR/AP).'}
          </p>
          <DataTable
            data={displayData as Record<string, unknown>[]}
            columns={columns}
            keyField={keyField}
            pageSize={50}
            maxHeight="65vh"
            onRowClick={handleRowClick}
            selectedRowKey={selectedRowKey}
          />
          {selectedRow && (
            <div className="subledger-accordion">
              <div className="subledger-accordion-header">
                <span className="subledger-accordion-title">
                  {activeTab === 'ar' && (selectedRow.invoice_number ?? selectedRow.document_id)}
                  {activeTab === 'ap' && (selectedRow.invoice_number ?? selectedRow.vendor_invoice_number)}
                  {activeTab === 'fa' && selectedRow.asset_number}
                  {activeTab === 'inventory' && selectedRow.material_id}
                  {activeTab === 'reconciliation' && 'Details'}
                  {!['ar', 'ap', 'fa', 'inventory', 'reconciliation'].includes(activeTab) && '—'}
                </span>
                <button
                  type="button"
                  className="subledger-accordion-close"
                  onClick={() => setSelectedRow(null)}
                  aria-label="Close"
                >
                  ×
                </button>
              </div>
              <div className="subledger-accordion-body">
                {hasLines ? (
                  <>
                    <h4 className="subledger-accordion-subtitle">
                      {lang === 'fr' ? `Lignes de facture (${lineRows.length})` : `Invoice lines (${lineRows.length})`}
                    </h4>
                    <DataTable
                      data={lineRows}
                      columns={invoiceLineColumns(lang)}
                      keyField="line_number"
                      pageSize={20}
                      maxHeight="280px"
                      pageSizeOptions={[]}
                    />
                  </>
                ) : (
                  <p className="subledger-accordion-no-lines">
                    {lang === 'fr'
                      ? (activeTab === 'ar' || activeTab === 'ap' ? 'Aucune ligne pour cet enregistrement.' : 'Détail des lignes pour les factures AR/AP. Sélectionnez une ligne ci-dessus.')
                      : (activeTab === 'ar' || activeTab === 'ap' ? 'No line items for this record.' : 'Line details are shown for AR/AP invoices. Select an invoice row above.')}
                  </p>
                )}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}
