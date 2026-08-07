import { useCallback, useEffect, useRef, useState } from 'react';
import { Graph } from '@antv/g6';
import { Button, Drawer, Form, Input, Spin, Tag } from 'antd';
import styles from './coop-graph-app.module.css';

/**
 * Knowledge graph spike app (SPEC-018 conventions):
 * - data-testid on interactive components (AI/E2E stable selectors)
 * - data-loading / data-state DOM reflection
 * - semantic markup + ARIA
 * - window.__APP_READY__ readiness flag
 * - structured JSON errors via window.dispatchEvent
 */

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:8080';

type DomainEvent = {
  type: 'entity_created' | 'property_set_saved';
  entity_id: string;
  kind?: string;
  instance_id?: string;
  scope?: string;
  version?: number;
  commit: string;
};

type Entity = { id: string; kind: string; name?: string };

function reportError(context: string, error: unknown) {
  // Structured JSON error output (AI-agent diagnosable)
  const payload = {
    context,
    message: error instanceof Error ? error.message : String(error),
    timestamp: new Date().toISOString(),
  };
  console.error(JSON.stringify({ '@type': 'app:Error', ...payload }));
  window.dispatchEvent(
    new CustomEvent('app:error', { detail: payload }),
  );
}

export function CoopGraphApp() {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphRef = useRef<Graph | null>(null);
  const [entities, setEntities] = useState<Record<string, Entity>>({});
  const [selected, setSelected] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [streamState, setStreamState] = useState<'connecting' | 'open' | 'error'>('connecting');

  // Boot the graph from the SSE genesis replay + keep it live.
  useEffect(() => {
    const graph = new Graph({
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
    graphRef.current = graph;

    const source = new EventSource(`${API_BASE}/events?cursor=`);
    let pending: DomainEvent[] = [];
    let flushTimer: ReturnType<typeof setTimeout> | null = null;

    const flush = () => {
      const next = { ...entitiesRef.current };
      for (const ev of pending) {
        if (ev.type === 'entity_created') {
          next[ev.entity_id] = { id: ev.entity_id, kind: ev.kind ?? 'Node' };
        }
      }
      pending = [];
      entitiesRef.current = next;
      setEntities(next);
      const nodes = Object.values(next).map((e) => ({ id: e.id, data: e }));
      graph.setData({ nodes, edges: [] });
      graph.render();
      if (flushTimer) clearTimeout(flushTimer);
      flushTimer = null;
    };

    source.onopen = () => {
      setStreamState('open');
      setLoading(false);
      // readiness flag for E2E agents (deterministic assertions)
      (window as any).__APP_READY__ = true;
    };
    source.onmessage = (msg) => {
      try {
        const ev = JSON.parse(msg.data) as DomainEvent;
        pending.push(ev);
        if (!flushTimer) flushTimer = setTimeout(flush, 50);
      } catch (e) {
        reportError('sse-parse', e);
      }
    };
    source.onerror = () => {
      setStreamState('error');
      reportError('sse', new Error('EventSource connection error'));
    };

    const entitiesRef = { current: entities };
    return () => {
      source.close();
      graph.destroy();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const onNodeClick = useCallback((ev: { target: { id: string } }) => {
    setSelected(ev.target.id);
  }, []);

  const selectedEntity = selected ? entities[selected] : undefined;

  return (
    <main className={styles.app} aria-label="Knowledge graph explorer">
      <header className={styles.header}>
        <h1>Network Graph</h1>
        <span>
          <Tag color={streamState === 'open' ? 'green' : streamState === 'error' ? 'red' : 'orange'}>
            stream: {streamState}
          </Tag>
          <Tag>{Object.keys(entities).length} entities</Tag>
        </span>
      </header>
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
      <Drawer
        title={selectedEntity ? `${selectedEntity.kind} — ${selectedEntity.id.slice(0, 8)}` : 'Entity'}
        open={!!selected}
        onClose={() => setSelected(null)}
        data-testid="entity-drawer"
        data-state={selected ? 'open' : 'closed'}
        aria-label="Entity details drawer"
      >
        {selectedEntity && <EntityForm entity={selectedEntity} key={selectedEntity.id} />}
      </Drawer>
    </main>
  );
}

function EntityForm({ entity }: { entity: Entity }) {
  const [properties, setProperties] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetch(`${API_BASE}/entities/${entity.id}/property-sets`)
      .then((r) => r.json())
      .then((body) => {
        if (cancelled) return;
        const items = body.items ?? [];
        if (items.length > 0) {
          const last = items[items.length - 1];
          const flat: Record<string, string> = {};
          for (const [k, v] of Object.entries(last.properties ?? {})) {
            flat[k] = String(v);
          }
          setProperties(flat);
        }
      })
      .catch((e) => reportError('property-sets-fetch', e));
    return () => {
      cancelled = true;
    };
  }, [entity.id]);

  const save = async (values: Record<string, string>) => {
    setSaving(true);
    setError(null);
    try {
      const instance = crypto.randomUUID();
      const body = {
        instance_id: instance,
        scope: 'Org',
        version: 1,
        properties: values,
      };
      const resp = await fetch(`${API_BASE}/entities/${entity.id}/property-sets`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'x-principal': 'spike-user' },
        body: JSON.stringify(body),
      });
      if (!resp.ok) {
        const err = await resp.json();
        throw new Error(err['api:message'] ?? 'save failed');
      }
      setProperties(values);
    } catch (e) {
      reportError('property-save', e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <Form
      layout="vertical"
      onFinish={save}
      initialValues={properties}
      key={JSON.stringify(properties)}
      data-testid="entity-form"
      data-saving={saving}
      aria-label={`Edit properties of ${entity.kind}`}
    >
      {error && <p role="alert" data-testid="entity-form-error">{error}</p>}
      <Form.Item label="name" name="name">
        <Input placeholder="entity name" data-testid="entity-name-input" />
      </Form.Item>
      <Form.Item>
        <Button type="primary" htmlType="submit" loading={saving} data-testid="entity-save-btn">
          Save
        </Button>
      </Form.Item>
    </Form>
  );
}
