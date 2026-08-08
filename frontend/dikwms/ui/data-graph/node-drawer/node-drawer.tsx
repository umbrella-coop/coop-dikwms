import { useEffect, useState } from 'react';
import { Button, Drawer, Form, Input } from 'antd';
import type { Entity } from '@coop-codes/dikwms.hook.use-data-graph-sse';
import { reportError } from '@coop-codes/dikwms.hook.use-data-graph-sse';

export type NodeDrawerProps = {
  entity: Entity | undefined;
  apiBase: string;
  onClose: () => void;
};

export function NodeDrawer({ entity, apiBase, onClose }: NodeDrawerProps) {
  const [properties, setProperties] = useState<Record<string, string>>({});
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!entity) return;
    let cancelled = false;
    setError(null);
    fetch(`${apiBase}/entities/${entity.id}/property-sets`)
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
  }, [entity, apiBase]);

  const save = async (values: Record<string, string>) => {
    if (!entity) return;
    setSaving(true);
    setError(null);
    try {
      const body = {
        instance_id: crypto.randomUUID(),
        scope: 'Org',
        version: 1,
        properties: values,
      };
      const resp = await fetch(`${apiBase}/entities/${entity.id}/property-sets`, {
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
    <Drawer
      title={entity ? `${entity.kind} — ${entity.id.slice(0, 8)}` : 'Entity'}
      open={!!entity}
      onClose={onClose}
      data-testid="node-drawer"
      data-state={entity ? 'open' : 'closed'}
      aria-label="Entity details drawer"
    >
      {entity && (
        <Form
          layout="vertical"
          onFinish={save}
          initialValues={properties}
          key={JSON.stringify(properties)}
          data-testid="node-form"
          data-saving={saving}
          aria-label={`Edit properties of ${entity.kind}`}
        >
          {error && (
            <p role="alert" data-testid="node-form-error">
              {error}
            </p>
          )}
          <Form.Item label="name" name="name">
            <Input placeholder="entity name" data-testid="node-name-input" />
          </Form.Item>
          <Form.Item>
            <Button
              type="primary"
              htmlType="submit"
              loading={saving}
              data-testid="node-save-btn"
            >
              Save
            </Button>
          </Form.Item>
        </Form>
      )}
    </Drawer>
  );
}
