import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { Graph } from './graph.js';

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

describe('SPEC-018 ui/graph', () => {
  const base = {
    entities: {},
    streamState: 'connecting' as const,
    loading: true,
    onSelect: () => {},
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  // AC-1: stable data-testid on the canvas
  it('exposes a stable data-testid on the canvas', () => {
    render(<Graph {...base} />);
    expect(screen.getByTestId('coop-graph-canvas')).toBeTruthy();
  });

  // AC-2: data-loading / data-state reflect async state
  it('reflects loading and stream state on the DOM', () => {
    const { rerender } = render(<Graph {...base} />);
    expect(screen.getByTestId('coop-graph-canvas').getAttribute('data-loading')).toBe('true');
    expect(screen.getByTestId('coop-graph-canvas').getAttribute('data-state')).toBe('connecting');

    rerender(<Graph {...base} loading={false} streamState="open" />);
    expect(screen.getByTestId('coop-graph-canvas').getAttribute('data-loading')).toBe('false');
    expect(screen.getByTestId('coop-graph-canvas').getAttribute('data-state')).toBe('open');
  });

  // AC-1: layout selector (user-facing layout control)
  it('exposes a layout selector', () => {
    render(<Graph {...base} />);
    expect(screen.getByTestId('layout-select')).toBeTruthy();
    expect(screen.getByLabelText('Graph layout type')).toBeTruthy();
  });

  // AC-1: entity count selector for E2E assertions
  it('exposes an entity count selector', () => {
    render(
      <Graph
        {...base}
        entities={{
          a: { id: 'a', kind: 'Node' },
          b: { id: 'b', kind: 'Edge' },
        }}
      />,
    );
    expect(screen.getByTestId('entity-count').textContent).toContain('2');
  });
});
