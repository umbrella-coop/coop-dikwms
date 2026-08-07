import { useEffect, useRef } from 'react';
import { Graph as G6Graph } from '@antv/g6';
import { Spin, Tag } from 'antd';
import type { Entity, EventStreamState } from '@coop-codes/network-graph.hooks.use-event-stream';
import styles from './graph.module.css';

export type GraphProps = {
  entities: Record<string, Entity>;
  streamState: EventStreamState;
  loading: boolean;
  onSelect: (id: string) => void;
};

export function Graph({ entities, streamState, loading, onSelect }: GraphProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<G6Graph | null>(null);
  const onSelectRef = useRef(onSelect);
  onSelectRef.current = onSelect;

  useEffect(() => {
    const graph = new G6Graph({
      container: containerRef.current!,
      autoFit: 'view',
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
        <Tag color={streamState === 'open' ? 'green' : streamState === 'error' ? 'red' : 'orange'}>
          stream: {streamState}
        </Tag>
        <Tag data-testid="entity-count">{Object.keys(entities).length} entities</Tag>
      </footer>
    </section>
  );
}
