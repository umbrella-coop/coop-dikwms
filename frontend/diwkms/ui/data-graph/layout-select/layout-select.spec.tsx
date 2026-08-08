import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { LayoutSelect, LAYOUT_OPTIONS } from './layout-select.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

it('renders the selector with the layout options', () => {
  render(<LayoutSelect value="force" onChange={() => {}} />);
  expect(screen.getByTestId('layout-select')).toBeTruthy();
  expect(screen.getByLabelText('Canvas layout type')).toBeTruthy();
});

it('calls onChange with the selected layout', () => {
  const onChange = vi.fn();
  render(<LayoutSelect value="force" onChange={onChange} />);
  fireEvent.mouseDown(screen.getByTestId('layout-select'));
  const option = screen.getByTitle(LAYOUT_OPTIONS[1].label);
  fireEvent.click(option);
  expect(onChange).toHaveBeenCalledTimes(1);
  expect(onChange.mock.calls[0][0]).toBe('d3-force');
});
