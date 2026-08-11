import { useEffect, useMemo, useState } from 'react';
import { Card, Drawer, Empty, Select, Slider, Spin, Tag, Typography } from 'antd';
import { Canvas } from '@coop-codes/dikwms.ui.data-graph-antv-g6.canvas';
import type { DataGraphEdge, DataGraphNode } from '@coop-codes/dikwms.type.core-v1';

/**
 * Git-domain explorer (SPEC-027 REQ-008/REQ-010): commit/author graph from
 * GET /graph/snapshot, timeline slider + author filters over the G6 canvas,
 * commit drawer with property-set history, and wisdom insight cards from the
 * per-repo git-insight entity (ADR-004: parent edges resolved client-side
 * from adjacency properties).
 */

export type GitExplorerProps = {
  apiBase: string;
};

type SnapshotEntity = {
  id: string;
  kind: string;
  property_sets: { properties: Record<string, unknown> }[];
};

type DirInsight = { dir: string } & Record<string, unknown>;

const { Text } = Typography;

export function GitExplorer({ apiBase }: GitExplorerProps) {
  const [entities, setEntities] = useState<SnapshotEntity[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<SnapshotEntity | null>(null);
  const [history, setHistory] = useState<unknown[] | null>(null);
  const [windowRange, setWindowRange] = useState<[number, number]>([0, 0]);
  const [authorFilter, setAuthorFilter] = useState<string[]>([]);

  const load = async () => {
    setLoading(true);
    try {
      const resp = await fetch(`${apiBase}/graph/snapshot`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = (await resp.json()) as { entities: SnapshotEntity[] };
      setEntities(data.entities ?? []);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [apiBase]);

  const commits = useMemo(
    () =>
      entities.filter((e) => {
        const hash = e.property_sets[0]?.properties?.hash;
        return typeof hash === 'string' && !hash.startsWith('git-insight-');
      }),
    [entities],
  );
  const authors = useMemo(
    () =>
      entities.filter((e) => {
        const email = e.property_sets[0]?.properties?.email;
        return typeof email === 'string';
      }),
    [entities],
  );
  const insightEntity = useMemo(
    () =>
      entities.find((e) =>
        String(e.property_sets[0]?.properties?.hash ?? '').startsWith('git-insight-'),
      ),
    [entities],
  );

  const timestamps = useMemo(
    () =>
      commits
        .map((c) => Date.parse(String(c.property_sets[0]?.properties?.authored_at ?? '')))
        .filter(Number.isFinite)
        .sort((a, b) => a - b),
    [commits],
  );

  useEffect(() => {
    if (timestamps.length > 0 && windowRange[1] === 0) {
      const min = timestamps[0];
      const max = timestamps[timestamps.length - 1];
      const span = max - min;
      const windowStart = span > 0 ? max - span * 0.2 : min;
      setWindowRange([windowStart, max]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [timestamps]);

  const visibleCommits = useMemo(() => {
    if (windowRange[1] === 0) return commits;
    return commits.filter((c) => {
      const t = Date.parse(String(c.property_sets[0]?.properties?.authored_at ?? ''));
      const inTime = t >= windowRange[0] && t <= windowRange[1];
      if (!inTime) return false;
      if (authorFilter.length === 0) return true;
      return authorFilter.includes(String(c.property_sets[0]?.properties?.author_email ?? ''));
    });
  }, [commits, windowRange, authorFilter]);

  const graph = useMemo(() => {
    const hashIndex = new Map<string, string>();
    for (const c of commits) {
      hashIndex.set(String(c.property_sets[0]?.properties?.hash), c.id);
    }
    const nodes: Record<string, DataGraphNode> = {};
    const edges: DataGraphEdge[] = [];
    for (const c of visibleCommits) {
      const props = c.property_sets[0]?.properties ?? {};
      const message = String(props.message ?? '').split('\n')[0].slice(0, 60);
      nodes[c.id] = { id: c.id, kind: 'Node', name: message || 'commit' };
      const parents = Array.isArray(props.parent_hexshas)
        ? (props.parent_hexshas as string[])
        : [];
      for (const parent of parents) {
        const target = hashIndex.get(parent);
        if (target) {
          edges.push({ id: `${c.id}->${target}`, kind: 'Edge', source: c.id, target });
        }
      }
      const authorId = String(props.author ?? '');
      if (authorId && nodes[authorId]) {
        edges.push({ id: `${c.id}->${authorId}`, kind: 'Edge', source: c.id, target: authorId });
      }
    }
    for (const a of authors) {
      const props = a.property_sets[0]?.properties ?? {};
      if (!nodes[a.id]) {
        nodes[a.id] = {
          id: a.id,
          kind: 'Node',
          name: String(props.name ?? props.email ?? 'author'),
        };
      }
    }
    return { nodes, edges };
  }, [visibleCommits, authors]);

  const insights = useMemo(() => {
    const raw = insightEntity?.property_sets[0]?.properties?.insights;
    if (!raw || typeof raw !== 'object') return [];
    return Object.entries(raw as Record<string, unknown>)
      .map(([dir, v]) => ({ dir, ...(v as Record<string, unknown>) }) as DirInsight)
      .sort((a, b) => {
        const risk = (x: DirInsight) => (x.bus_factor_risk ? 1 : 0);
        return risk(b) - risk(a);
      });
  }, [insightEntity]);

  const authorOptions = useMemo(
    () =>
      authors.map((a) => ({
        value: String(a.property_sets[0]?.properties?.email ?? ''),
        label: String(
          a.property_sets[0]?.properties?.name ?? a.property_sets[0]?.properties?.email ?? '',
        ),
      })),
    [authors],
  );

  const openDrawer = (id: string) => {
    const entity = entities.find((e) => e.id === id);
    setSelected(entity ?? null);
    setHistory(null);
    if (entity) {
      fetch(`${apiBase}/entities/${id}/property-sets`)
        .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
        .then((d) => setHistory(Array.isArray(d) ? d : []))
        .catch(() => setHistory([]));
    }
  };

  const riskyInsights = insights.filter((i) => i.bus_factor_risk);
  const staleInsights = insights.filter((i) => i.stale && !i.bus_factor_risk);

  if (error) {
    return (
      <Empty
        data-testid="git-explorer-error"
        description={
          <span>
            Failed to load the git graph: <Text code>{error}</Text>
          </span>
        }
      />
    );
  }

  return (
    <div
      className="git-explorer"
      data-testid="git-explorer"
      data-loading={loading}
      data-error={error !== null}
    >
      <div data-testid="git-explorer-controls" className="git-explorer-controls">
        <Select
          mode="multiple"
          allowClear
          placeholder="Filter by author"
          options={authorOptions}
          value={authorFilter}
          onChange={(v: string[]) => setAuthorFilter(v)}
          data-testid="git-explorer-author-filter"
          style={{ minWidth: 240 }}
        />
        <Slider
          range
          min={timestamps[0] ?? 0}
          max={timestamps[timestamps.length - 1] ?? 1}
          value={windowRange}
          onChange={(v) => setWindowRange(v as [number, number])}
          data-testid="git-explorer-timeline"
          style={{ flex: 1 }}
        />
        <Tag data-testid="git-explorer-count">
          {visibleCommits.length} / {commits.length} commits
        </Tag>
      </div>

      <div style={{ display: 'flex', gap: 16 }}>
        <div style={{ flex: 1, minHeight: 480, position: 'relative' }}>
          {loading ? (
            <Spin data-testid="git-explorer-spinner" />
          ) : (
            <Canvas
              entities={graph.nodes}
              edges={graph.edges}
              streamState="open"
              loading={false}
              onSelect={openDrawer}
            />
          )}
        </div>
        <aside style={{ width: 280 }} data-testid="git-explorer-insights">
          <Typography.Title level={5}>Wisdom insights</Typography.Title>
          {riskyInsights.length === 0 && staleInsights.length === 0 && (
            <Text type="secondary">No risk flags on this window.</Text>
          )}
          {riskyInsights.slice(0, 3).map((i) => (
            <Card
              key={i.dir}
              size="small"
              title={i.dir}
              extra={<Tag color="red">bus-factor</Tag>}
              data-testid={`insight-card-${i.dir}`}
            >
              <Text>
                {String(i.dominant_author)} owns {(Number(i.dominant_share) * 100).toFixed(0)}% —
                no activity in the last 90 days.
              </Text>
            </Card>
          ))}
          {staleInsights.slice(0, 3).map((i) => (
            <Card
              key={i.dir}
              size="small"
              title={i.dir}
              extra={<Tag color="orange">stale</Tag>}
              data-testid={`insight-card-${i.dir}`}
            >
              <Text>Dormant zone: last activity {String(i.last_active ?? 'unknown')}.</Text>
            </Card>
          ))}
        </aside>
      </div>

      <Drawer
        open={selected !== null}
        onClose={() => setSelected(null)}
        title={selected ? String(selected.property_sets[0]?.properties?.hash ?? selected.id) : ''}
        data-testid="git-explorer-drawer"
      >
        {selected && (
          <div data-testid="git-explorer-drawer-meta">
            <pre data-testid="git-explorer-drawer-props">
              {JSON.stringify(selected.property_sets[0]?.properties ?? {}, null, 2)}
            </pre>
            <Typography.Title level={5}>Property-set history</Typography.Title>
            {history === null ? (
              <Spin size="small" />
            ) : history.length === 0 ? (
              <Text type="secondary">No property-set versions.</Text>
            ) : (
              <ul data-testid="git-explorer-drawer-history">
                {history.map((v, i) => (
                  <li key={i}>
                    <Text code>
                      v{String((v as { version?: unknown }).version ?? i + 1)} —{' '}
                      {String((v as { status?: unknown }).status ?? '')}
                    </Text>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </Drawer>
    </div>
  );
}
