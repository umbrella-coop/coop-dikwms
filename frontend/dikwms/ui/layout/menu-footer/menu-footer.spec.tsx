import React from 'react';
import { render, screen } from '@testing-library/react';
import { LayoutMenuFooter } from './menu-footer.js';

it('renders the footer lines', () => {
  render(<LayoutMenuFooter lines={['© 2026 dikwms', 'Built with bit']} />);
  expect(screen.getByTestId('layout-menu-footer')).toBeTruthy();
  expect(screen.getByText('© 2026 dikwms')).toBeTruthy();
  expect(screen.getByText('Built with bit')).toBeTruthy();
});
