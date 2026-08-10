import { useState } from 'react';
import { useDataGraphSse } from '@coop-codes/dikwms.hook.use-data-graph-sse';
import { Canvas } from '@coop-codes/dikwms.ui.data-graph-antv-g6.canvas';
import { NodeDrawer } from '@coop-codes/dikwms.ui.data-graph.node-drawer';
import { LayoutFrame } from '@coop-codes/dikwms.ui.layout.frame';
import { LayoutHeaderTitle } from '@coop-codes/dikwms.ui.layout.header-title';
import { LayoutSearch } from '@coop-codes/dikwms.ui.layout.search';
import { LayoutHeaderActions } from '@coop-codes/dikwms.ui.layout.header-actions';
import { LayoutUserMenu } from '@coop-codes/dikwms.ui.layout.user-menu';
import { LayoutMenuFooter } from '@coop-codes/dikwms.ui.layout.menu-footer';
import styles from './app.module.css';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8080';

const MENU = [
  {
    key: 'explore',
    label: 'Explore',
    children: [
      { key: 'graph', label: 'Live graph' },
      { key: 'entities', label: 'Entities' },
    ],
  },
];

export function App() {
  const { entities, streamState, ready } = useDataGraphSse(API_BASE);
  const [selected, setSelected] = useState<string | null>(null);
  const [selectedKey, setSelectedKey] = useState('graph');
  const [collapsed, setCollapsed] = useState(false);
  const selectedEntity = selected ? entities[selected] : undefined;

  return (
    <LayoutFrame
      layout="side"
      fixSider
      menu={MENU}
      selectedKey={selectedKey}
      onMenuSelect={setSelectedKey}
      collapsed={collapsed}
      onCollapsedChange={setCollapsed}
      headerTitle={<LayoutHeaderTitle title="dikwms" />}
      headerActions={
        <>
          <LayoutSearch placeholder="Search entities" />
          <LayoutHeaderActions
            items={[
              { key: 'info', icon: <span>i</span>, tooltip: 'Info' },
              { key: 'help', icon: <span>?</span>, tooltip: 'Help' },
            ]}
          />
          <LayoutUserMenu title="Orchestrator" items={[{ key: 'logout', label: 'Logout' }]} />
        </>
      }
      siderFooter={<LayoutMenuFooter lines={['© 2026 dikwms', 'DIKW Management System']} />}
    >
      <main className={styles.app} aria-label="Knowledge graph explorer" data-ready={ready}>
        <Canvas
          entities={entities}
          streamState={streamState}
          loading={!ready}
          onSelect={setSelected}
        />
        <NodeDrawer entity={selectedEntity} apiBase={API_BASE} onClose={() => setSelected(null)} />
      </main>
    </LayoutFrame>
  );
}
