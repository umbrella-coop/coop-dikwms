import React from 'react';
import { render, screen } from '@testing-library/react';
import { ProjectPageWraper } from './view-wrapper.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

it('renders the live view container for viewType live', () => {
  render(<ProjectPageWraper viewType="live" title="Project" subTitle="demo" />);
  expect(screen.getByTestId('project-live-view-container')).toBeTruthy();
  expect(screen.getByText('Project')).toBeTruthy();
});

it('renders the tabular placeholder for viewType tabular', () => {
  render(<ProjectPageWraper viewType="tabular" title="Project" />);
  expect(screen.getByTestId('project-view-wrapper-tabular-placeholder')).toBeTruthy();
});
