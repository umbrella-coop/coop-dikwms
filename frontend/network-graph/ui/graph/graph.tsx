import { useEffect, useRef, useState } from 'react';
import { Graph as G6Graph } from '@antv/g6';
import { Select, Spin, Tag } from 'antd';
import type { Entity, EventStreamState } from '@coop-codes/network-graph.hooks.use-event-stream';
import styles from './graph.module.css';

export type GraphLayout = { type: string; [key: string]: unknown };

const LAYOUT_OPTIONS: { value: string; label: string }[] = [
  { value: 'force', label: 'Force (default)' },
  { value: 'd3-force', label: 'D3 Force' },
  { value: 'fruchterman', label: 'Fruchterman' },
  { value: 'grid', label: 'Grid' },
  { value: 'circular', label: 'Circular' },
  { value: 'radial', label: 'Radial' },
  { value: 'concentric', label: 'Concentric' },
  { value: 'mds', label: 'MDS' },
  { value: 'random', label: 'Random' },
];

/** User-friendly default: force-directed with overlap prevention. */
const DEFAULT_LAYOUT: GraphLayout = {
  type: 'force',
  gravity: 10,
  linkDistance: 120,
  preventOverlap: true,
  enableWorker: true,
};

export type GraphProps = {
  entities: Record<string, Entity>;
  streamState: EventStreamState;
  loading: boolean;
  onSelect: (id: string) => void;
  layout?: GraphLayout;
};

export function Graph({ entities, streamState, loading, onSelect, layout = DEFAULT_LAYOUT }: GraphProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<G6Graph | null>(null);
  const onSelectRef = useRef(onSelect);
  onSelectRef.current = onSelect;
  const [layoutType, setLayoutType] = useState(layout.type);
  const layoutRef = useRef<GraphLayout>(layout);
  layoutRef.current = layout;

  useEffect(() => {
    const graph = new G6Graph({
      container: containerRef.current!,
      autoFit: 'view',
      layout: layoutRef.current,
      node: {
        style: {
          labelText: (d) => {
            const e = d.data as unknown as Entity;
            return e.name ?? e.id.slice(0, 8);
          },
          labelPlacement: 'bottom',
        },
      },
      behaviors: ['drag-canvas', 'zoom-canvas', 'click-select'],
    });
    graph.on('node:click', (ev) => {
      const id = (ev as unknown as { target?: { id?: string } }).target?.id;
      if (id) onSelectRef.current(id);
    });
    graphRef.current = graph;
    return () => {
      graph.destroy();
      graphRef.current = null;
    };
  }, []);

  useEffect(() => {
    const graph = graphRef.current;
    if (!graph) return;
    const nodes = Object.values(entities).map((e) => ({ id: e.id, data: e }));
    graph.setData({ nodes, edges: [] });
    graph.render();
  }, [entities]);

  // layout switch: re-run the layout algorithm on the current data
  useEffect(() => {
    const graph = graphRef.current;
    if (!graph) return;
    graph.setLayout({ ...layoutRef.current, type: layoutType });
    graph.layout();
  }, [layoutType]);

  return (
    <section className={styles.wrap}>
      <div
        ref={containerRef}
        className={styles.canvas}
        data-testid="coop-graph-canvas"
        data-loading={loading}
        data-state={streamState}
        role="img"
        aria-label="Knowledge graph visualization canvas"
      >
        {loading && <Spin data-testid="graph-loading" />}
      </div>
      <footer className={styles.footer}>
        <Select
          value={layoutType}
          onChange={setLayoutType}
          options={LAYOUT_OPTIONS}
          size="small"
          data-testid="layout-select"
          aria-label="Graph layout type"
        />
        <Tag color={streamState === 'open' ? 'green' : streamState === 'error' ? 'red' : 'orange'}>
          stream: {streamState}
        </Tag>
        <Tag data-testid="entity-count">{Object.keys(entities).length} entities</Tag>
      </footer>
    </section>
  );
}
