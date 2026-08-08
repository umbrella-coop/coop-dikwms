import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { LayoutFrame } from './frame.js';

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  }),
});

it('renders sider, header and content slots', () => {
  render(
    <LayoutFrame
      headerTitle={<span data-testid="title-slot">dikwms</span>}
      headerActions={<span data-testid="actions-slot">act</span>}
      siderFooter={<span data-testid="footer-slot">©</span>}
      menu={[{ key: 'graph', label: 'Graph' }]}
    >
      <div data-testid="content-slot">content</div>
    </LayoutFrame>,
  );
  expect(screen.getByTestId('layout-frame')).toBeTruthy();
  expect(screen.getByTestId('title-slot')).toBeTruthy();
  expect(screen.getByTestId('actions-slot')).toBeTruthy();
  expect(screen.getByTestId('footer-slot')).toBeTruthy();
  expect(screen.getByTestId('content-slot')).toBeTruthy();
  expect(screen.getByText('Graph')).toBeTruthy();
});

it('reflects layout and collapsed state on the DOM and fires menu select', () => {
  const onMenuSelect = vi.fn();
  const { rerender } = render(
    <LayoutFrame layout="side" collapsed menu={[{ key: 'a', label: 'A' }, { key: 'b', label: 'B' }]} onMenuSelect={onMenuSelect} />,
  );
  expect(screen.getByTestId('layout-frame').getAttribute('data-layout')).toBe('side');
  expect(screen.getByTestId('layout-frame').getAttribute('data-collapsed')).toBe('true');
  fireEvent.click(screen.getByText('B'));
  expect(onMenuSelect).toHaveBeenCalledWith('b');
  rerender(<LayoutFrame layout="top" menu={[]} />);
  expect(screen.getByTestId('layout-frame').getAttribute('data-layout')).toBe('top');
});
