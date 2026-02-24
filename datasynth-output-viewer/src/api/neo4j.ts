/**
 * Neo4j connection and graph data loading for the Graph view.
 * Uses the official neo4j-driver to run Cypher and return nodes/links for visualization.
 */

import neo4j, { type Driver, type Session } from 'neo4j-driver';

const STORAGE_URI = 'datasynth-graph-neo4j-uri';
const STORAGE_USER = 'datasynth-graph-neo4j-user';

export interface GraphNode {
  id: string;
  label: string;
  name?: string;
  code?: string;
  [key: string]: unknown;
}

export interface GraphLink {
  source: string;
  target: string;
  type?: string;
  /** Total flow (sum of all flows on this edge). */
  amount?: number;
  /** Number of flow instances (transactions/document pairs). */
  count?: number;
  /** Mean flow = amount / count. */
  mean_flow?: number;
  /** Min single flow. */
  min_flow?: number;
  /** Max single flow. */
  max_flow?: number;
  [key: string]: unknown;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

let driver: Driver | null = null;

function nodeId(n: { elementId?: string; identity?: { toString?: () => string } }): string {
  if (n && typeof (n as { elementId?: string }).elementId === 'string') return (n as { elementId: string }).elementId;
  if (n && (n as { identity?: unknown }).identity != null) {
    const id = (n as { identity: { toString?: () => string } }).identity;
    if (typeof id.toString === 'function') return id.toString();
  }
  return String(Math.random());
}

function getLabels(n: { labels?: string[] }): string {
  const labels = n.labels;
  if (Array.isArray(labels) && labels.length > 0) return labels[0];
  return 'Node';
}

function getProperties(n: { properties?: Record<string, unknown> }): Record<string, unknown> {
  const p = n.properties;
  if (p && typeof p === 'object') return { ...p };
  return {};
}

/**
 * Create a Neo4j driver (connection). Does not verify connectivity.
 */
export function createDriver(uri: string, user: string, password: string): Driver {
  const normalizedUri = uri.trim() || 'bolt://localhost:7687';
  return neo4j.driver(normalizedUri, neo4j.auth.basic(user.trim() || 'neo4j', password));
}

/**
 * Close the current driver and clear it.
 */
export function closeDriver(): void {
  if (driver) {
    try {
      driver.close();
    } catch {
      /* ignore */
    }
    driver = null;
  }
}

/**
 * Get or create the global driver. Call connect() first.
 */
export function getDriver(): Driver | null {
  return driver;
}

/**
 * Connect to Neo4j and store the driver. Returns true if verification succeeds.
 */
export async function connect(uri: string, user: string, password: string): Promise<boolean> {
  closeDriver();
  try {
    driver = createDriver(uri, user, password);
    await driver.verifyConnectivity();
    try {
      localStorage.setItem(STORAGE_URI, uri.trim() || 'bolt://localhost:7687');
      localStorage.setItem(STORAGE_USER, user.trim() || 'neo4j');
    } catch {
      /* ignore */
    }
    return true;
  } catch {
    driver = null;
    return false;
  }
}

/**
 * Load graph data from Neo4j using a Cypher query that returns (n)-[r]->(m).
 * Default query: MATCH (n)-[r]->(m) RETURN n, type(r) AS relType, r, m LIMIT 500
 */
export async function loadGraphFromNeo4j(
  limit = 500,
  customQuery?: string
): Promise<GraphData> {
  const d = getDriver();
  if (!d) throw new Error('Not connected to Neo4j. Enter URI and credentials and click Connect.');

  const query =
    customQuery ||
    `MATCH (n)-[r]->(m)
     RETURN n, type(r) AS relType, r, m
     LIMIT ${Math.max(1, Math.min(limit, 2000))}`;

  const nodesMap = new Map<string, GraphNode>();
  const links: GraphLink[] = [];

  const session: Session = d.session();

  try {
    const result = await session.run(query);

    for (const record of result.records) {
      const n = record.get('n');
      const m = record.get('m');
      const relType = record.get('relType');
      const r = record.get('r');

      if (!n || !m) continue;

      const idN = nodeId(n as { identity?: { toString?: () => string }; elementId?: string });
      const idM = nodeId(m as { identity?: { toString?: () => string }; elementId?: string });

      if (!nodesMap.has(idN)) {
        const props = getProperties(n as { properties?: Record<string, unknown> });
        const label = getLabels(n as { labels?: string[] });
        nodesMap.set(idN, {
          id: idN,
          label: label,
          name: (props.name as string) ?? (props.code as string) ?? idN,
          code: props.code as string,
          ...props,
        });
      }
      if (!nodesMap.has(idM)) {
        const props = getProperties(m as { properties?: Record<string, unknown> });
        const label = getLabels(m as { labels?: string[] });
        nodesMap.set(idM, {
          id: idM,
          label: label,
          name: (props.name as string) ?? (props.code as string) ?? idM,
          code: props.code as string,
          ...props,
        });
      }

      const link: GraphLink = { source: idN, target: idM };
      if (relType != null) link.type = String(relType);
      if (r && typeof r === 'object' && 'properties' in r && r.properties) {
        const rp = (r as { properties: Record<string, unknown> }).properties;
        if (typeof rp.amount === 'number') link.amount = rp.amount;
        if (typeof rp.amount === 'string') link.amount = parseFloat(rp.amount);
      }
      links.push(link);
    }

    return {
      nodes: Array.from(nodesMap.values()),
      links: deduplicateLinks(links),
    };
  } finally {
    await session.close();
  }
}

/**
 * Load saved Neo4j URI and user from localStorage (password is never stored).
 */
export function getStoredConnection(): { uri: string; user: string } {
  try {
    return {
      uri: localStorage.getItem(STORAGE_URI) || 'bolt://localhost:7687',
      user: localStorage.getItem(STORAGE_USER) || 'neo4j',
    };
  } catch {
    return { uri: 'bolt://localhost:7687', user: 'neo4j' };
  }
}

/**
 * One edge per account pair: collapse (A,B) and (B,A) into a single edge (unordered pair).
 * Merge flow stats when multiple links exist for the same pair.
 */
export function deduplicateLinks(links: GraphLink[]): GraphLink[] {
  const byPair = new Map<string, GraphLink>();
  for (const l of links) {
    const s = String(l.source);
    const t = String(l.target);
    const key = s <= t ? `${s}\t${t}` : `${t}\t${s}`;
    const source = s <= t ? s : t;
    const target = s <= t ? t : s;
    const existing = byPair.get(key);
    if (!existing) {
      byPair.set(key, { ...l, source, target });
      continue;
    }
    const amount = (existing.amount as number) ?? 0;
    const count = (existing.count as number) ?? 0;
    const a = (l.amount as number) ?? 0;
    const c = (l.count as number) ?? 0;
    const newAmount = amount + a;
    const newCount = count + c;
    const newMean = newCount > 0 ? newAmount / newCount : 0;
    const existingMin = (existing.min_flow as number) ?? a;
    const existingMax = (existing.max_flow as number) ?? a;
    const linkMin = (l.min_flow as number) ?? a;
    const linkMax = (l.max_flow as number) ?? a;
    byPair.set(key, {
      ...existing,
      source,
      target,
      amount: Math.round(newAmount * 100) / 100,
      count: newCount,
      mean_flow: Math.round(newMean * 100) / 100,
      min_flow: Math.min(existingMin, linkMin),
      max_flow: Math.max(existingMax, linkMax),
    });
  }
  return Array.from(byPair.values());
}

/**
 * Parse whitelist string (comma or newline separated) into a set of normalized node IDs.
 * Empty or whitespace-only input returns an empty set (no filtering).
 */
export function parseWhitelist(text: string): Set<string> {
  const set = new Set<string>();
  const parts = text.split(/[\s,\n]+/);
  for (const p of parts) {
    const id = p.trim();
    if (id) set.add(id);
  }
  return set;
}

/**
 * Filter graph to only whitelisted nodes and their direct connections (related document flows).
 * Empty whitelist = full graph (no filtering). Non-empty: keep nodes in whitelist or adjacent (1-hop) and their links.
 */
export function filterGraphByWhitelist(data: GraphData, whitelist: Set<string>): GraphData {
  if (whitelist.size === 0) return { nodes: data.nodes, links: deduplicateLinks(data.links) };

  const keepIds = new Set<string>();
  for (const node of data.nodes) {
    if (whitelist.has(node.id)) keepIds.add(node.id);
  }
  for (const link of data.links) {
    const s = String(link.source);
    const t = String(link.target);
    if (whitelist.has(s) || whitelist.has(t)) {
      keepIds.add(s);
      keepIds.add(t);
    }
  }

  const nodes = data.nodes.filter((n) => keepIds.has(n.id));
  const filteredLinks = data.links.filter(
    (l) => keepIds.has(String(l.source)) && keepIds.has(String(l.target))
  );
  return { nodes, links: deduplicateLinks(filteredLinks) };
}
