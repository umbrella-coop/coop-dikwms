import React from 'react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { GitExplorer } from './git-explorer.js';

vi.mock('@antv/g6', () => ({
  Graph: class {
    on() {}
    setData() {}
    setLayout() {}
    layout() {}
    render() {}
    destroy() {}
  },
}));

vi.mock('@coop-codes/dikwms.ui.data-graph-antv-g6.canvas', () => ({
  Canvas: (props: { onSelect: (id: string) => void; edges: unknown[] }) => (
    <button data-testid="fake-canvas" onClick={() => props.onSelect('commit-1')}>
      canvas ({props.edges.length} edges)
    </button>
  ),
}));

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

const SNAPSHOT = {
  entities: [
    {
      id: 'commit-1',
      kind: 'Node',
      property_sets: [
        {
          properties: {
            hash: 'aaa111',
            message: 'fix: thing\n\nbody',
            authored_at: '2026-01-05T00:00:00+00:00',
            author: 'author-1',
            author_email: 'alice@example.com',
            parent_hexshas: [],
            paths: ['src/x.rs'],
          },
        },
      ],
    },
    {
      id: 'commit-2',
      kind: 'Node',
      property_sets: [
        {
          properties: {
            hash: 'bbb222',
            message: 'feat: more',
            authored_at: '2026-01-06T00:00:00+00:00',
            author: 'author-1',
            author_email: 'alice@example.com',
            parent_hexshas: ['aaa111'],
            paths: ['src/y.rs'],
          },
        },
      ],
    },
    {
      id: 'author-1',
      kind: 'Node',
      property_sets: [{ properties: { name: 'Alice', email: 'alice@example.com' } }],
    },
    {
      id: 'insight-1',
      kind: 'Node',
      property_sets: [
        {
          properties: {
            hash: 'git-insight-repo-2025-08-11',
            insights: {
              src: {
                commits: 2,
                authors: 1,
                dominant_author: 'alice@example.com',
                dominant_share: 1,
                stale: true,
                bus_factor_risk: true,
              },
            },
          },
        },
      ],
    },
  ],
};

function mockFetch() {
  globalThis.fetch = vi.fn(async (url: RequestInfo | URL) => {
    const path = String(url);
    if (path.endsWith('/graph/snapshot')) {
      return { ok: true, json: async () => SNAPSHOT } as Response;
    }
    if (path.includes('/entities/commit-1/property-sets')) {
      return { ok: true, json: async () => [{ version: 1, status: 'Current' }] } as Response;
    }
    throw new Error(`unexpected fetch ${path}`);
  }) as unknown as typeof fetch;
}

describe('SPEC-027 ui/project/git-explorer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch();
  });

  it('renders the explorer with commit count, timeline and author filter', async () => {
    render(<GitExplorer apiBase="http://test" />);
    await waitFor(() => expect(screen.getByTestId('git-explorer-count')).toBeTruthy());
    expect(screen.getByTestId('git-explorer-count').textContent).toMatch(/\/ 2 commits/);
    // antd Slider/Select expose ARIA roles, not data-testid (SPEC-018 targets
    // our own components; antd wrappers assert via roles)
    expect(screen.getAllByRole('slider').length).toBeGreaterThan(0);
    expect(screen.getByRole('combobox')).toBeTruthy();
  });

  it('passes parent edges to the canvas (ADR-004 client-side resolution)', async () => {
    render(<GitExplorer apiBase="http://test" />);
    await waitFor(() => expect(screen.getByTestId('fake-canvas')).toBeTruthy());
    expect(screen.getByTestId('fake-canvas').textContent).toContain('1 edges');
  });

  it('renders wisdom insight cards from the git-insight entity', async () => {
    render(<GitExplorer apiBase="http://test" />);
    await waitFor(() => expect(screen.getByTestId('git-explorer-insights')).toBeTruthy());
    expect(screen.getByTestId('insight-card-src')).toBeTruthy();
    expect(screen.getByText('bus-factor')).toBeTruthy();
  });

  it('opens the drawer with property-set history on node selection', async () => {
    render(<GitExplorer apiBase="http://test" />);
    await waitFor(() => expect(screen.getByTestId('fake-canvas')).toBeTruthy());
    fireEvent.click(screen.getByTestId('fake-canvas'));
    await waitFor(() => expect(screen.getByTestId('git-explorer-drawer-meta')).toBeTruthy());
    expect(screen.getByText('aaa111')).toBeTruthy();
    await waitFor(() =>
      expect(screen.getByTestId('git-explorer-drawer-history').textContent).toContain('v1'),
    );
  });

  it('surfaces fetch errors', async () => {
    globalThis.fetch = vi.fn(async () => {
      throw new Error('boom');
    }) as unknown as typeof fetch;
    render(<GitExplorer apiBase="http://test" />);
    await waitFor(() => expect(screen.getByTestId('git-explorer-error')).toBeTruthy());
  });
});
