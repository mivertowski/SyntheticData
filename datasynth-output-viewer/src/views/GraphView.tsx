import { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import {
  connect,
  closeDriver,
  loadGraphFromNeo4j,
  getStoredConnection,
  getDriver,
  parseWhitelist,
  filterGraphByWhitelist,
  type GraphData,
} from '../api/neo4j';
import { loadGraphInMemory, buildGraphFromJournalEntries, DEFAULT_ACCOUNT_MAX_DEPTH } from '../api/graphInMemory';
import type { JournalEntryRow } from '../types';
import './GraphView.css';

const DEFAULT_LIMIT = 500;

type GraphSource = 'memory' | 'neo4j';

export function GraphView({ resolvedTheme = 'dark' }: { resolvedTheme?: 'light' | 'dark' }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<{
    graphData: (d: GraphData) => void;
    zoomToFit: (ms?: number, padding?: number) => void;
    backgroundColor: (color: string) => void;
  } | null>(null);

  const [source, setSource] = useState<GraphSource>('memory');
  const [uri, setUri] = useState('');
  const [user, setUser] = useState('');
  const [password, setPassword] = useState('');
  const [connected, setConnected] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [graphData, setGraphData] = useState<GraphData | null>(null);
  const [rawJeData, setRawJeData] = useState<{ rows: JournalEntryRow[]; coaNames: Map<string, string> } | null>(null);
  const [whitelistText, setWhitelistText] = useState('');
  const [includeAuxiliaryAccounts, setIncludeAuxiliaryAccounts] = useState(false);
  const [accountMaxDepth, setAccountMaxDepth] = useState(DEFAULT_ACCOUNT_MAX_DEPTH);
  const [limit, setLimit] = useState(DEFAULT_LIMIT);

  const whitelistSet = useMemo(() => parseWhitelist(whitelistText), [whitelistText]);
  const displayData = useMemo(() => {
    if (rawJeData) {
      return buildGraphFromJournalEntries(rawJeData.rows, rawJeData.coaNames, {
        whitelist: whitelistSet,
        includeAuxiliary: includeAuxiliaryAccounts,
        accountMaxDepth: accountMaxDepth >= 1 && accountMaxDepth <= 7 ? accountMaxDepth : undefined,
      });
    }
    if (!graphData) return null;
    return filterGraphByWhitelist(graphData, whitelistSet);
  }, [rawJeData, graphData, whitelistSet, includeAuxiliaryAccounts, accountMaxDepth]);

  // Restore stored URI/user on mount
  useEffect(() => {
    const stored = getStoredConnection();
    setUri(stored.uri);
    setUser(stored.user);
  }, []);

  const graphBgColor = resolvedTheme === 'light' ? '#ffffff' : '#1e1e1e';

  // Init force-graph when container is ready
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    import('force-graph').then((module) => {
      const ForceGraph = module.default;
      const graph = new ForceGraph(el);
      graph
        .backgroundColor(graphBgColor)
        .nodeLabel((n: unknown) => {
          const o = n as Record<string, unknown>;
          return (o?.code ?? o?.id ?? '') as string;
        })
        .nodeAutoColorBy('label')
        .linkAutoColorBy('type')
        .linkDirectionalArrowLength(0)
        .linkDirectionalParticles(0)
        .linkLabel((l: unknown) => {
          const o = l as Record<string, unknown>;
          const fmt = (n: number) => n.toLocaleString('en-US', { maximumFractionDigits: 2 });
          const parts: string[] = [];
          if (o?.type) parts.push(String(o.type));
          if (typeof o?.count === 'number') parts.push(`n=${o.count}`);
          if (typeof o?.amount === 'number') parts.push(`total=${fmt(o.amount as number)}`);
          if (typeof o?.mean_flow === 'number') parts.push(`mean=${fmt(o.mean_flow as number)}`);
          if (typeof o?.min_flow === 'number') parts.push(`min=${fmt(o.min_flow as number)}`);
          if (typeof o?.max_flow === 'number') parts.push(`max=${fmt(o.max_flow as number)}`);
          return parts.length ? parts.join(' · ') : '';
        });
      graphRef.current = graph;
      return () => {
        graphRef.current = null;
      };
    });
  }, []);

  // Sync canvas background when theme changes
  useEffect(() => {
    const graph = graphRef.current;
    if (graph) graph.backgroundColor(graphBgColor);
  }, [graphBgColor]);

  // Cleanup driver on unmount
  useEffect(() => {
    return () => {
      closeDriver();
    };
  }, []);

  // Update graph when display data or theme changes
  useEffect(() => {
    const graph = graphRef.current;
    if (!graph || !displayData) return;
    graph.backgroundColor(graphBgColor);
    graph.graphData(displayData);
    if (displayData.nodes.length > 0) {
      setTimeout(() => graph.zoomToFit(400, 40), 100);
    }
  }, [displayData, graphBgColor]);

  const handleConnect = useCallback(async () => {
    setError(null);
    setLoading(true);
    try {
      const ok = await connect(uri, user, password);
      setConnected(ok);
      if (!ok) setError('Connection failed. Check URI, username, and password.');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Connection failed');
      setConnected(false);
    } finally {
      setLoading(false);
    }
  }, [uri, user, password]);

  const handleDisconnect = useCallback(() => {
    closeDriver();
    setConnected(false);
    setGraphData(null);
    setRawJeData(null);
    setError(null);
  }, []);

  const handleLoadInMemory = useCallback(async () => {
    setSource('memory');
    setGraphData(null);
    setError(null);
    setLoading(true);
    try {
      const { rows, coaNames } = await loadGraphInMemory();
      setRawJeData({ rows, coaNames });
      if (rows.length === 0) {
        setError('No journal entries in data. Load output data (journal_entries.csv) first.');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to build graph from data');
      setRawJeData(null);
    } finally {
      setLoading(false);
    }
  }, []);

  const handleLoadFromNeo4j = useCallback(async () => {
    if (!getDriver()) {
      setError('Connect to Neo4j first.');
      return;
    }
    setSource('neo4j');
    setRawJeData(null);
    setError(null);
    setLoading(true);
    try {
      const data = await loadGraphFromNeo4j(limit);
      setGraphData(data);
      if (data.nodes.length === 0 && data.links.length === 0) {
        setError('No nodes or relationships returned. Run the generator with graph export (Neo4j) and import the data into Neo4j.');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load graph');
      setGraphData(null);
    } finally {
      setLoading(false);
    }
  }, [limit]);

  return (
    <div className="graph-view">
      <h2>Graph Export</h2>
      <p className="graph-desc">
        Double-entry bookkeeping has a natural directed-graph representation: GL accounts as nodes, debit→credit flows as edges.
        Use <strong>in-memory</strong> (from journal entries in this viewer) or connect to <strong>Neo4j</strong> when you have exported graph data there.
      </p>

      <div className="graph-actions">
        <div className="graph-actions-primary">
          <button
            type="button"
            className="graph-btn graph-btn-inmemory"
            onClick={handleLoadInMemory}
            disabled={loading}
          >
            {loading && source === 'memory' ? 'Building…' : 'Load graph (in-memory)'}
          </button>
          <span className="graph-actions-hint">From journal_entries.csv + chart of accounts — no server</span>
        </div>

        <div className="graph-connect">
          <span className="graph-connect-label">Or Neo4j server:</span>
          <div className="graph-connect-fields">
            <label>
              <span>URI</span>
              <input
                type="text"
                value={uri}
                onChange={(e) => setUri(e.target.value)}
                placeholder="bolt://localhost:7687"
                disabled={connected}
              />
            </label>
            <label>
              <span>User</span>
              <input
                type="text"
                value={user}
                onChange={(e) => setUser(e.target.value)}
                placeholder="neo4j"
                disabled={connected}
              />
            </label>
            <label>
              <span>Password</span>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
                disabled={connected}
              />
            </label>
          </div>
          <div className="graph-connect-actions">
            {!connected ? (
              <button type="button" className="graph-btn graph-btn-connect" onClick={handleConnect} disabled={loading}>
                {loading ? 'Connecting…' : 'Connect'}
              </button>
            ) : (
              <>
                <button type="button" className="graph-btn graph-btn-load" onClick={handleLoadFromNeo4j} disabled={loading}>
                  {loading && source === 'neo4j' ? 'Loading…' : 'Load from Neo4j'}
                </button>
                <label className="graph-limit">
                  Limit
                  <input
                    type="number"
                    min={50}
                    max={2000}
                    step={50}
                    value={limit}
                    onChange={(e) => setLimit(Number(e.target.value) || DEFAULT_LIMIT)}
                  />
                </label>
                <button type="button" className="graph-btn graph-btn-disconnect" onClick={handleDisconnect}>
                  Disconnect
                </button>
              </>
            )}
          </div>
        </div>
      </div>

      {error && <p className="graph-error">{error}</p>}

      {(graphData || rawJeData) && (
        <>
          <div className="graph-whitelist-wrap">
            <label htmlFor="graph-whitelist" className="graph-whitelist-label">
              Whitelist accounts
            </label>
            <textarea
              id="graph-whitelist"
              className="graph-whitelist-input"
              placeholder="e.g. 4000, 2000, 1100 — one per line or comma-separated. Empty = all."
              value={whitelistText}
              onChange={(e) => setWhitelistText(e.target.value)}
              rows={2}
              aria-label="Account codes; only entries containing at least one of these accounts are shown (in-memory)"
            />
            <p className="graph-whitelist-hint">
              {rawJeData
                ? 'Empty whitelist = full graph (all entries). With account codes: only entries containing at least one whitelisted account are included; all accounts and flows from those entries are shown.'
                : 'Empty whitelist = full graph. With account codes: only whitelisted nodes and their direct connections are shown.'}
            </p>
          </div>
          {rawJeData && (
            <>
              <div className="graph-max-depth-wrap">
                <label htmlFor="graph-account-max-depth" className="graph-max-depth-label">
                  Account max depth
                </label>
                <input
                  id="graph-account-max-depth"
                  type="number"
                  min={1}
                  max={7}
                  value={accountMaxDepth}
                  onChange={(e) => setAccountMaxDepth(Math.min(7, Math.max(1, Number(e.target.value) || DEFAULT_ACCOUNT_MAX_DEPTH)))}
                  className="graph-max-depth-input"
                  aria-label="Account max depth (digits)"
                />
                <span className="graph-max-depth-hint">
                  French GAAP: truncate account to this many digits (default 3 = class/subclass, up to 7).
                </span>
              </div>
              <div className="graph-auxiliary-wrap">
                <label>
                  <input
                    type="checkbox"
                    checked={includeAuxiliaryAccounts}
                    onChange={(e) => setIncludeAuxiliaryAccounts(e.target.checked)}
                    aria-label="Include auxiliary accounts"
                  />
                  <span>Include auxiliary accounts</span>
                </label>
                <p className="graph-auxiliary-hint">
                  Adds nodes for compte auxiliaire (e.g. 401xxxx, 411xxxx) and links from GL to auxiliary.
                </p>
              </div>
            </>
          )}
          <p className="graph-stats">
            {displayData?.nodes.length ?? 0} nodes, {displayData?.links.length ?? 0} edges (one per account pair).
            {rawJeData && whitelistSet.size > 0 && ' Entries containing whitelist.'}
            {graphData && whitelistSet.size > 0 && ` Filtered from ${graphData.nodes.length} nodes.`}
            Drag to pan, scroll to zoom, drag nodes to rearrange.
          </p>
        </>
      )}

      <div
        ref={containerRef}
        className="graph-canvas"
        style={{ width: '100%', height: (graphData || rawJeData) ? '70vh' : '320px', minHeight: '320px' }}
      />
      {!graphData && !rawJeData && !loading && (
        <p className="graph-hint">
          Click <strong>Load graph (in-memory)</strong> to build the transaction graph from the loaded journal entries (no Neo4j required).
        </p>
      )}
    </div>
  );
}
