import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { LayoutSearch } from './search.js';

it('renders the search input and reflects value changes', () => {
  const onChange = vi.fn();
  render(<LayoutSearch placeholder="搜索方案" value="" onChange={onChange} />);
  const input = screen.getByTestId('layout-search-input') as HTMLInputElement;
  fireEvent.change(input, { target: { value: 'graph' } });
  expect(onChange).toHaveBeenCalledWith('graph');
});

it('renders and fires the quick action', () => {
  const onActionClick = vi.fn();
  render(<LayoutSearch placeholder="搜索" actionIcon={<span>+</span>} onActionClick={onActionClick} />);
  fireEvent.click(screen.getByTestId('layout-search-action'));
  expect(onActionClick).toHaveBeenCalledTimes(1);
});
