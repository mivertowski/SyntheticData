/**
 * Build a transaction graph in memory from journal entries (no Neo4j server).
 * Double-entry representation: GL accounts as nodes, debit→credit flows as directed edges.
 */

import { loadJournalEntriesCsv, loadChartOfAccounts } from './data';
import type { JournalEntryRow } from '../types';
import type { GraphData, GraphNode, GraphLink } from './neo4j';

export interface GraphBuildOptions {
  /** When non-empty, only include journal entries that have at least one line with gl_account (or auxiliary) in this set. */
  whitelist?: Set<string>;
  /** When true, add nodes for auxiliary_account_number and links GL → auxiliary. */
  includeAuxiliary?: boolean;
  /** Max depth (digits) for account codes: truncate to first N digits (French GAAP: 3 = class/subclass, up to 7). 0 or undefined = no truncation. */
  accountMaxDepth?: number;
}

export const DEFAULT_ACCOUNT_MAX_DEPTH = 2;
const MIN_ACCOUNT_MAX_DEPTH = 1;
const MAX_ACCOUNT_MAX_DEPTH = 7;

function truncateAccountCode(code: string, depth: number): string {
  if (!depth || depth < 1) return code;
  return code.slice(0, Math.min(depth, MAX_ACCOUNT_MAX_DEPTH));
}

function toNum(v: string | number | null | undefined): number {
  if (v == null) return 0;
  const n = typeof v === 'number' ? v : parseFloat(String(v));
  return Number.isFinite(n) ? n : 0;
}

const AUX_PREFIX = 'aux_';

/** French PCG: fournisseurs (401, 408…) and clients (411…). Not 400 (sales). When includeAuxiliary we skip these as endpoints so we get shortcut edges (other ↔ aux) only. */
function isVendorOrCustomerGl(gl: string): boolean {
  if (gl.startsWith('41')) return true;
  if (gl.length >= 3 && gl.startsWith('40') && gl.charAt(2) !== '0') return true;
  return false;
}

/** True if this node id is a fournisseurs/clients aggregate to exclude when includeAuxiliary (shortcut: no 40/41 nodes). */
function isVendorOrCustomerNode(id: string): boolean {
  if (id === '41' || id === '40') return true;
  if (id.startsWith('41')) return true;
  if (id.startsWith('40') && (id.length <= 2 || id.charAt(2) !== '0')) return true;
  return false;
}

/**
 * Build graph from flat journal entry lines.
 * - Edge = at least one journal entry (document) involves both accounts (debit on one, credit on the other).
 * - Exactly one edge per account pair (unordered); no redundant edges.
 * - Only COA account numbers for GL nodes; non-COA codes are excluded.
 * - accountMaxDepth (1–7): truncate account codes to that many digits (French GAAP: 3 = class/subclass, up to 7); reduces nodes/edges by aggregating.
 * - When includeAuxiliary: shortcut connections only — lines with aux use aux as endpoint; vendor/customer lines without aux are skipped so we never create (401/411↔aux). No 40/41 nodes; only direct edges (other account ↔ aux).
 */
export function buildGraphFromJournalEntries(
  rows: JournalEntryRow[],
  coaNames: Map<string, string>,
  options?: GraphBuildOptions
): GraphData {
  const whitelist = options?.whitelist;
  const includeAuxiliary = options?.includeAuxiliary === true;
  const maxDepth = options?.accountMaxDepth;
  const depth = maxDepth != null && maxDepth >= MIN_ACCOUNT_MAX_DEPTH && maxDepth <= MAX_ACCOUNT_MAX_DEPTH ? maxDepth : 0;
  const truncate = (code: string) => (depth ? truncateAccountCode(code, depth) : code);

  /** No whitelist or empty whitelist = all entries in the graph. Non-empty whitelist = only entries containing at least one whitelisted account. */
  const useWhitelist = whitelist != null && whitelist.size > 0;

  let workingRows = rows;

  if (useWhitelist) {
    const byDoc = new Map<string, JournalEntryRow[]>();
    for (const r of rows) {
      const id = r.document_id ?? '';
      if (!id) continue;
      if (!byDoc.has(id)) byDoc.set(id, []);
      byDoc.get(id)!.push(r);
    }
    const docIdsWithWhitelist = new Set<string>();
    for (const [docId, lines] of byDoc) {
      const hasMatch = lines.some((line) => {
        const gl = (line.gl_account ?? '').trim();
        if (gl && (whitelist.has(gl) || (depth && whitelist.has(truncate(gl))))) return true;
        if (includeAuxiliary) {
          const aux = (line.auxiliary_account_number ?? '').toString().trim();
          if (aux && whitelist.has(aux)) return true;
        }
        return false;
      });
      if (hasMatch) docIdsWithWhitelist.add(docId);
    }
    workingRows = rows.filter((r) => docIdsWithWhitelist.has(r.document_id ?? ''));
  }

  /** Only include GL accounts that exist in the chart of accounts (no non-COA account numbers). */
  const validCoaGl =
    coaNames.size > 0 ? new Set<string>(coaNames.keys()) : null;

  const nodeIds = new Set<string>();
  /** Per (source,target): aggregate flow stats — no redundant edges, one edge per pair with cardinality props. */
  interface FlowStats {
    sum: number;
    count: number;
    min: number;
    max: number;
  }
  const linkStats = new Map<string, FlowStats>();
  const auxLabels = new Map<string, string>();

  const byDoc = new Map<string, JournalEntryRow[]>();
  for (const r of workingRows) {
    const id = r.document_id ?? '';
    if (!id) continue;
    if (!byDoc.has(id)) byDoc.set(id, []);
    byDoc.get(id)!.push(r);
  }

  for (const [, lines] of byDoc) {
    const debits: { gl: string; amount: number }[] = [];
    const credits: { gl: string; amount: number }[] = [];
    for (const line of lines) {
      // When includeAuxiliary: use aux as endpoint so we create direct edges (other_account, aux) from this JE.
      const gl = (line.gl_account ?? '').trim();
      if (!gl) continue;
      const inCoa = validCoaGl == null || validCoaGl.size === 0 || validCoaGl.has(gl);
      const allowForShortcut = includeAuxiliary && !isVendorOrCustomerGl(gl);
      if (!inCoa && !allowForShortcut) continue;
      const glNode = truncate(gl);
      const debit = toNum(line.debit_amount);
      const credit = toNum(line.credit_amount);
      const aux = (line.auxiliary_account_number ?? '').toString().trim();
      if (includeAuxiliary && isVendorOrCustomerGl(gl) && !aux) continue;
      const useAuxAsEndpoint = includeAuxiliary && aux.length > 0;

      if (useAuxAsEndpoint) {
        const auxNode = AUX_PREFIX + aux;
        nodeIds.add(auxNode);
        if (!auxLabels.has(auxNode)) {
          auxLabels.set(auxNode, (line.auxiliary_account_label ?? aux) as string);
        }
        if (debit > 0) {
          debits.push({ gl: auxNode, amount: debit });
        }
        if (credit > 0) {
          credits.push({ gl: auxNode, amount: credit });
        }
      } else {
        if (debit > 0) {
          debits.push({ gl: glNode, amount: debit });
          nodeIds.add(glNode);
        }
        if (credit > 0) {
          credits.push({ gl: glNode, amount: credit });
          nodeIds.add(glNode);
        }
      }
    }
    for (const d of debits) {
      for (const c of credits) {
        if (d.gl === c.gl) continue;
        const key = `${d.gl}\t${c.gl}`;
        const flow = Math.min(d.amount, c.amount);
        const prev = linkStats.get(key);
        if (!prev) {
          linkStats.set(key, { sum: flow, count: 1, min: flow, max: flow });
        } else {
          prev.sum += flow;
          prev.count += 1;
          prev.min = Math.min(prev.min, flow);
          prev.max = Math.max(prev.max, flow);
        }
      }
    }
  }

  function coaNameForTruncated(truncatedCode: string): string {
    const direct = coaNames.get(truncatedCode);
    if (direct) return direct;
    for (const [fullCode, name] of coaNames) {
      if (fullCode.startsWith(truncatedCode)) return name;
    }
    return truncatedCode;
  }

  const nodes: GraphNode[] = [];
  for (const id of nodeIds) {
    if (includeAuxiliary && isVendorOrCustomerNode(id)) continue;
    if (id.startsWith(AUX_PREFIX)) {
      const code = id.slice(AUX_PREFIX.length);
      nodes.push({
        id,
        label: 'Auxiliary',
        name: auxLabels.get(id) ?? code,
        code,
      });
    } else {
      nodes.push({
        id,
        label: 'Account',
        name: coaNameForTruncated(id),
        code: id,
      });
    }
  }

  const round2 = (x: number) => Math.round(x * 100) / 100;

  /** One edge per unordered account pair: key is canonical (min, max) so (A,B) and (B,A) become a single edge. */
  const linkByPair = new Map<string, GraphLink>();

  function canonicalKey(source: string, target: string): string {
    return source <= target ? `${source}\t${target}` : `${target}\t${source}`;
  }

  function orderedPair(source: string, target: string): [string, string] {
    return source <= target ? [source, target] : [target, source];
  }

  function addOrMergeLink(
    source: string,
    target: string,
    type: string,
    sum: number,
    count: number,
    min: number,
    max: number
  ) {
    const key = canonicalKey(source, target);
    const [s, t] = orderedPair(source, target);
    const existing = linkByPair.get(key);
    if (!existing) {
      const mean = count > 0 ? sum / count : 0;
      linkByPair.set(key, {
        source: s,
        target: t,
        type,
        amount: round2(sum),
        count,
        mean_flow: round2(mean),
        min_flow: round2(min),
        max_flow: round2(max),
      });
    } else {
      const existingAmount = (existing.amount as number) ?? 0;
      const existingCount = (existing.count as number) ?? 0;
      const existingMin = (existing.min_flow as number) ?? min;
      const existingMax = (existing.max_flow as number) ?? max;
      const newSum = existingAmount + sum;
      const newCount = existingCount + count;
      const newMean = newCount > 0 ? newSum / newCount : 0;
      linkByPair.set(key, {
        source: existing.source as string,
        target: existing.target as string,
        type: existing.type ?? type,
        amount: round2(newSum),
        count: newCount,
        mean_flow: round2(newMean),
        min_flow: round2(Math.min(existingMin, min)),
        max_flow: round2(Math.max(existingMax, max)),
      });
    }
  }

  for (const [key, stats] of linkStats) {
    const [source, target] = key.split('\t');
    addOrMergeLink(source, target, 'FLOW', stats.sum, stats.count, stats.min, stats.max);
  }

  let links = Array.from(linkByPair.values());
  if (includeAuxiliary) {
    links = links.filter(
      (l) => !isVendorOrCustomerNode(String(l.source)) && !isVendorOrCustomerNode(String(l.target))
    );
  }
  return { nodes, links };
}

/**
 * Load journal entries and chart of accounts. Returns raw data so the viewer can rebuild the graph when whitelist or includeAuxiliary change.
 */
export async function loadGraphInMemory(): Promise<{
  rows: JournalEntryRow[];
  coaNames: Map<string, string>;
}> {
  const [rows, coa] = await Promise.all([
    loadJournalEntriesCsv(),
    loadChartOfAccounts(),
  ]);

  const coaNames = new Map<string, string>();
  coa.forEach((v, k) => coaNames.set(k, v.name));

  return { rows, coaNames };
}
