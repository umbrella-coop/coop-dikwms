import React from 'react';
import { render, screen } from '@testing-library/react';
import { DataGraphProjectInteractiveCanvasContainer } from './project-interactive-canvas-container.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

it('renders header (title, subTitle, extra) and children', () => {
  render(
    <DataGraphProjectInteractiveCanvasContainer title="Project" subTitle="demo" extra={<button data-testid="page-action">Save</button>}>
      <div data-testid="page-children">content</div>
    </DataGraphProjectInteractiveCanvasContainer>,
  );
  expect(screen.getByTestId('project-interactive-canvas-container')).toBeTruthy();
  expect(screen.getByText('Project')).toBeTruthy();
  expect(screen.getByText('demo')).toBeTruthy();
  expect(screen.getByTestId('page-action')).toBeTruthy();
  expect(screen.getByTestId('page-children')).toBeTruthy();
});

it('renders breadcrumb, tabs and footer', () => {
  render(
    <DataGraphProjectInteractiveCanvasContainer
      breadcrumb={[{ title: 'Home' }, { title: 'Project' }]}
      tabs={{ items: [{ key: 'graph', label: 'Graph' }] }}
      footer={<div data-testid="page-footer">footer</div>}
    />,
  );
  expect(screen.getByText('Home')).toBeTruthy();
  expect(screen.getByText('Graph')).toBeTruthy();
  expect(screen.getByTestId('page-footer')).toBeTruthy();
});

it('reflects loading on the DOM', () => {
  const { rerender } = render(<DataGraphProjectInteractiveCanvasContainer loading />);
  expect(screen.getByTestId('project-interactive-canvas-container').getAttribute('data-loading')).toBe('true');
  expect(screen.getByTestId('project-interactive-canvas-container-loading')).toBeTruthy();
  rerender(<DataGraphProjectInteractiveCanvasContainer loading={false} />);
  expect(screen.getByTestId('project-interactive-canvas-container').getAttribute('data-loading')).toBe('false');
});
