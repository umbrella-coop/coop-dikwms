import { useEffect, useRef, useState } from 'react';
import { Graph as G6Graph } from '@antv/g6';
import { Spin, Tag } from 'antd';
import { LayoutSelect } from '@coop-codes/dikwms.ui.data-graph-antv-g6.layout-select';
import type { EventStreamState } from '@coop-codes/dikwms.hook.use-data-graph-sse';
import type { DataGraphEdge, DataGraphNode, GraphNodeShape } from '@coop-codes/dikwms.type.core-v1';
import styles from './canvas.module.css';

/** Per-shape palette (SPEC-027): git entities are visually distinguishable. */
const SHAPE_COLORS: Record<GraphNodeShape, string> = {
  node: '#868e96',
  commit: '#1971c2',
  author: '#2f9e44',
  repository: '#e8590c',
  organization: '#9c36b5',
  insight: '#f08c00',
};

/** G6 style callbacks receive the node model — custom data lives in `.data`. */
function nodeShape(d: unknown): GraphNodeShape {
  const model = d as { data?: DataGraphNode };
  return model.data?.shape ?? 'node';
}

export type CanvasLayout = { type: string; [key: string]: unknown };

/** User-friendly default: force-directed with overlap prevention.
 * enableWorker: false — G6 worker layout breaks under vite dev (worker script
 * resolves to HTML; options closures fail structured clone; fallback crashes
 * with 'postLayout' undefined). Main-thread layout is fine at this scale. */
const DEFAULT_LAYOUT: CanvasLayout = {
  type: 'force',
  gravity: 10,
  linkDistance: 120,
  preventOverlap: true,
  enableWorker: false,
};

export type CanvasProps = {
  entities: Record<string, DataGraphNode>;
  streamState: EventStreamState;
  loading: boolean;
  onSelect: (id: string) => void;
  layout?: CanvasLayout;
  /** Optional graph edges (SPEC-027: git parent links built client-side from
   * adjacency properties, ADR-004). */
  edges?: DataGraphEdge[];
  /** Hide the in-footer layout select — the parent renders its own (e.g. in
   * a controls row next to other filters). */
  hideLayoutSelect?: boolean;
};

export function Canvas({
  entities,
  streamState,
  loading,
  onSelect,
  layout = DEFAULT_LAYOUT,
  edges = [],
  hideLayoutSelect = false,
}: CanvasProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<G6Graph | null>(null);
  const onSelectRef = useRef(onSelect);
  onSelectRef.current = onSelect;
  const [layoutType, setLayoutType] = useState(layout.type);
  const layoutRef = useRef<CanvasLayout>(layout);
  layoutRef.current = layout;

  // External control: a parent may drive the layout type (e.g. its own select
  // in a controls row). Re-apply whenever the prop type changes.
  useEffect(() => {
    setLayoutType(layout.type);
  }, [layout.type]);

  useEffect(() => {
    const canvas = new G6Graph({
      container: containerRef.current!,
      autoFit: 'view',
      autoResize: true,
      layout: layoutRef.current,
      node: {
        style: {
          fill: (d: unknown) => SHAPE_COLORS[nodeShape(d)],
          labelText: (d) => {
            const e = d.data as unknown as DataGraphNode;
            return e.name ?? e.id.slice(0, 8);
          },
          labelPlacement: 'bottom',
          labelFill: '#333',
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
    canvas.setData({ nodes, edges });
    canvas.render();
  }, [entities, edges]);

  // layout switch: re-run the layout algorithm on the current data.
  // Skips the initial mount — render() already lays out from construction
  // options; a second concurrent layout() races G6's layout state machine
  // ('postLayout' undefined crash).
  const mountedRef = useRef(false);
  useEffect(() => {
    if (!mountedRef.current) {
      mountedRef.current = true;
      return;
    }
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
        {!hideLayoutSelect && <LayoutSelect value={layoutType} onChange={setLayoutType} />}
        <Tag color={streamState === 'open' ? 'green' : streamState === 'error' ? 'red' : 'orange'}>
          stream: {streamState}
        </Tag>
        <Tag data-testid="entity-count">{Object.keys(entities).length} entities</Tag>
      </footer>
    </section>
  );
}
