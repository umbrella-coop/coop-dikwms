import { useState } from 'react';
import { useEventStream } from '@coop-codes/network-graph.hooks.use-event-stream';
import { Graph } from '@coop-codes/network-graph.ui.graph';
import { EntityDrawer } from '@coop-codes/network-graph.ui.entity-drawer';
import styles from './coop-graph-app.module.css';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8080';

export function CoopGraphApp() {
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
