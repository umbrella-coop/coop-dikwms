import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach, beforeAll } from 'vitest';

beforeAll(() => {
  // antd v6 needs matchMedia in jsdom
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
});
import { NodeDrawer } from './node-drawer.js';

const entity = { id: '019f-entity', kind: 'Node', name: 'acme' };

describe('SPEC-018 ui/node-drawer', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ items: [] }),
    } as Response);
  });

  // AC-1: stable selectors on drawer, form, and inputs
  it('exposes stable data-testid selectors', async () => {
    render(<NodeDrawer entity={entity} apiBase="http://api" onClose={() => {}} />);
    expect(screen.getByTestId('node-drawer')).toBeTruthy();
    await waitFor(() => expect(screen.getByTestId('node-form')).toBeTruthy());
    expect(screen.getByTestId('node-name-input')).toBeTruthy();
    expect(screen.getByTestId('node-save-btn')).toBeTruthy();
  });

  // AC-2: data-state reflects drawer open/closed
  it('reflects open state on the drawer', () => {
    const { rerender } = render(<NodeDrawer entity={entity} apiBase="http://api" onClose={() => {}} />);
    expect(screen.getByTestId('node-drawer').getAttribute('data-state')).toBe('open');
    rerender(<NodeDrawer entity={undefined} apiBase="http://api" onClose={() => {}} />);
    expect(screen.getByTestId('node-drawer').getAttribute('data-state')).toBe('closed');
  });
});
