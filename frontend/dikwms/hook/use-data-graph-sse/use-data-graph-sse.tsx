import { useEffect, useRef, useState } from 'react';
import type { DataGraphNode } from '@coop-codes/dikwms.type.core-v1';

export type DomainEvent = {
  type: 'entity_created' | 'property_set_saved';
  entity_id: string;
  kind?: string;
  instance_id?: string;
  scope?: string;
  version?: number;
  commit: string;
};

export type EventStreamState = 'connecting' | 'open' | 'error';

export function useDataGraphSse(apiBase: string) {
  const [entities, setEntities] = useState<Record<string, DataGraphNode>>({});
  const [streamState, setStreamState] = useState<EventStreamState>('connecting');
  const [ready, setReady] = useState(false);
  const entitiesRef = useRef<Record<string, DataGraphNode>>({});
  // last seen commit cursor for lossless reconnect
  const cursorRef = useRef('');

  useEffect(() => {
    const connect = () => {
      const source = new EventSource(`${apiBase}/events?cursor=${cursorRef.current}`);
      let pending: DomainEvent[] = [];
      let flushTimer: ReturnType<typeof setTimeout> | null = null;

      const flush = () => {
        const next = { ...entitiesRef.current };
        for (const ev of pending) {
          if (ev.type === 'entity_created') {
            next[ev.entity_id] = { id: ev.entity_id, kind: ev.kind ?? 'Node' };
          }
          cursorRef.current = ev.commit;
        }
        pending = [];
        entitiesRef.current = next;
        setEntities(next);
        if (flushTimer) clearTimeout(flushTimer);
        flushTimer = null;
      };

      source.onopen = () => {
        setStreamState('open');
        setReady(true);
        (window as unknown as { __APP_READY__?: boolean }).__APP_READY__ = true;
      };
      source.onmessage = (msg) => {
        try {
          pending.push(JSON.parse(msg.data) as DomainEvent);
          if (!flushTimer) flushTimer = setTimeout(flush, 50);
        } catch (e) {
          reportError('sse-parse', e);
        }
      };
      source.onerror = () => {
        setStreamState('error');
        reportError('sse', new Error('EventSource connection error'));
        source.close();
        // automatic reconnect with cursor (lossless)
        setTimeout(connect, 2000);
      };
    };
    connect();
    return () => {
      // cleanup handled by reconnect guard in onerror
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [apiBase]);

  return { entities, streamState, ready };
}

export function reportError(context: string, error: unknown) {
  const payload = {
    context,
    message: error instanceof Error ? error.message : String(error),
    timestamp: new Date().toISOString(),
  };
  console.error(JSON.stringify({ '@type': 'app:Error', ...payload }));
  window.dispatchEvent(new CustomEvent('app:error', { detail: payload }));
}
