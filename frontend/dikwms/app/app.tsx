import { useState } from 'react';
import { useEventStream } from '@coop-codes/dikwms.hook.use-data-graph-sse';
import { Graph } from '@coop-codes/dikwms.ui.data-graph.canvas';
import { EntityDrawer } from '@coop-codes/dikwms.ui.data-graph.node-drawer';
import styles from './app.module.css';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8080';

export function App() {
  const { entities, streamState, ready } = useEventStream(API_BASE);
  const [selected, setSelected] = useState<string | null>(null);
  const selectedEntity = selected ? entities[selected] : undefined;

  return (
    <main className={styles.app} aria-label="Knowledge graph explorer" data-ready={ready}>
      <header className={styles.header}>
        <h1>Network Graph</h1>
      </header>
      <Graph
        entities={entities}
        streamState={streamState}
        loading={!ready}
        onSelect={setSelected}
      />
      <EntityDrawer entity={selectedEntity} apiBase={API_BASE} onClose={() => setSelected(null)} />
    </main>
  );
}
