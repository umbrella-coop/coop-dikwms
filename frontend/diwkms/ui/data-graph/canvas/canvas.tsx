import { useEffect, useRef, useState } from 'react';
import { Canvas as G6Canvas } from '@antv/g6';
import { Select, Spin, Tag } from 'antd';
import type { Entity, EventStreamState } from '@coop-codes/diwkms.hook.use-data-graph-sse';
import styles from './canvas.module.css';

export type CanvasLayout = { type: string; [key: string]: unknown };

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
const DEFAULT_LAYOUT: CanvasLayout = {
  type: 'force',
  gravity: 10,
  linkDistance: 120,
  preventOverlap: true,
  enableWorker: true,
};

export type CanvasProps = {
  entities: Record<string, Entity>;
  streamState: EventStreamState;
  loading: boolean;
  onSelect: (id: string) => void;
  layout?: CanvasLayout;
};

export function Canvas({ entities, streamState, loading, onSelect, layout = DEFAULT_LAYOUT }: CanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<G6Canvas | null>(null);
  const onSelectRef = useRef(onSelect);
  onSelectRef.current = onSelect;
  const [layoutType, setLayoutType] = useState(layout.type);
  const layoutRef = useRef<CanvasLayout>(layout);
  layoutRef.current = layout;

  useEffect(() => {
    const canvas = new G6Canvas({
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
    canvas.on('node:click', (ev) => {
      const id = (ev as unknown as { target?: { id?: string } }).target?.id;
      if (id) onSelectRef.current(id);
    });
    canvasRef.current = canvas;
    return () => {
      canvas.destroy();
      canvasRef.current = null;
    };
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const nodes = Object.values(entities).map((e) => ({ id: e.id, data: e }));
    canvas.setData({ nodes, edges: [] });
    canvas.render();
  }, [entities]);

  // layout switch: re-run the layout algorithm on the current data
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    canvas.setLayout({ ...layoutRef.current, type: layoutType });
    canvas.layout();
  }, [layoutType]);

  return (
    <section className={styles.wrap}>
      <div
        ref={containerRef}
        className={styles.canvas}
        data-testid="coop-canvas-canvas"
        data-loading={loading}
        data-state={streamState}
        role="img"
        aria-label="Knowledge canvas visualization canvas"
      >
        {loading && <Spin data-testid="canvas-loading" />}
      </div>
      <footer className={styles.footer}>
        <Select
          value={layoutType}
          onChange={setLayoutType}
          options={LAYOUT_OPTIONS}
          size="small"
          data-testid="layout-select"
          aria-label="Canvas layout type"
        />
        <Tag color={streamState === 'open' ? 'green' : streamState === 'error' ? 'red' : 'orange'}>
          stream: {streamState}
        </Tag>
        <Tag data-testid="entity-count">{Object.keys(entities).length} entities</Tag>
      </footer>
    </section>
  );
}
