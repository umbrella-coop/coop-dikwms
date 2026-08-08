import { useState } from 'react';
import { useDataGraphSse } from '@coop-codes/dikwms.hook.use-data-graph-sse';
import { Canvas } from '@coop-codes/dikwms.ui.data-graph-antv-g6.canvas';
import { NodeDrawer } from '@coop-codes/dikwms.ui.data-graph.node-drawer';
import styles from './app.module.css';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8080';

export function App() {
  const { entities, streamState, ready } = useDataGraphSse(API_BASE);
  const [selected, setSelected] = useState<string | null>(null);
  const selectedEntity = selected ? entities[selected] : undefined;

  return (
    <main className={styles.app} aria-label="Knowledge graph explorer" data-ready={ready}>
      <header className={styles.header}>
        <h1>Network Graph</h1>
      </header>
      <Canvas
        entities={entities}
        streamState={streamState}
        loading={!ready}
        onSelect={setSelected}
      />
      <NodeDrawer entity={selectedEntity} apiBase={API_BASE} onClose={() => setSelected(null)} />
    </main>
  );
}
