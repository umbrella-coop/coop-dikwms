import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { LayoutUserMenu } from './user-menu.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

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

it('renders the avatar with title', () => {
  render(<LayoutUserMenu title="书琰" items={[{ key: 'logout', label: '退出登录' }]} />);
  expect(screen.getByTestId('layout-user-menu')).toBeTruthy();
  expect(screen.getByText('书琰')).toBeTruthy();
});

it('opens the menu and fires onSelect', () => {
  const onSelect = vi.fn();
  render(<LayoutUserMenu title="书琰" items={[{ key: 'logout', label: '退出登录' }]} onSelect={onSelect} />);
  fireEvent.click(screen.getByTestId('layout-user-menu'));
  fireEvent.click(screen.getByText('退出登录'));
  expect(onSelect).toHaveBeenCalledWith('logout');
});
