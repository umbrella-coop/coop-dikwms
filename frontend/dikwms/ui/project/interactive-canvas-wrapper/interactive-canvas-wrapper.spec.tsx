import React from 'react';
import { render, screen } from '@testing-library/react';
import { ProjectLiveViewWraper } from './interactive-canvas-wrapper.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

it('renders the live view container for viewType graph', () => {
  render(<ProjectLiveViewWraper viewType="graph" title="Project" subTitle="demo" />);
  expect(screen.getByTestId('project-live-view-container')).toBeTruthy();
  expect(screen.getByText('Project')).toBeTruthy();
});

it('renders placeholders for future view types', () => {
  render(<ProjectLiveViewWraper viewType="tabular" title="Project" />);
  expect(screen.getByTestId('interactive-canvas-wrapper-tabular-placeholder')).toBeTruthy();
  render(<ProjectLiveViewWraper viewType="geographic" title="Project" />);
  expect(screen.getByTestId('interactive-canvas-wrapper-geographic-placeholder')).toBeTruthy();
});
