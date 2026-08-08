import React from 'react';
import { render, screen } from '@testing-library/react';
import { LayoutHeaderTitle } from './header-title.js';

it('renders logo, title and extra', () => {
  render(<LayoutHeaderTitle logo="logo.png" title="dikwms" extra={<span data-testid="extra">mega</span>} />);
  expect(screen.getByTestId('layout-header-title')).toBeTruthy();
  expect(screen.getByText('dikwms')).toBeTruthy();
  expect(screen.getByTestId('extra')).toBeTruthy();
  expect((screen.getByTestId('layout-header-title').querySelector('img') as HTMLImageElement).src).toContain('logo.png');
});

it('calls onClick', () => {
  const onClick = vi.fn();
  render(<LayoutHeaderTitle title="dikwms" onClick={onClick} />);
  screen.getByTestId('layout-header-title').click();
  expect(onClick).toHaveBeenCalledTimes(1);
});
