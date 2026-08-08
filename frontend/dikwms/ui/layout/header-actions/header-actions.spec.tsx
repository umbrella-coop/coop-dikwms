import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { LayoutHeaderActions } from './header-actions.js';

it('renders actions with per-item testids and fires onClick', () => {
  const onClick = vi.fn();
  render(
    <LayoutHeaderActions
      items={[
        { key: 'info', icon: <span>i</span>, tooltip: 'Info' },
        { key: 'github', icon: <span>g</span>, onClick },
      ]}
    />,
  );
  expect(screen.getByTestId('layout-header-actions-info')).toBeTruthy();
  fireEvent.click(screen.getByTestId('layout-header-actions-github'));
  expect(onClick).toHaveBeenCalledTimes(1);
});
