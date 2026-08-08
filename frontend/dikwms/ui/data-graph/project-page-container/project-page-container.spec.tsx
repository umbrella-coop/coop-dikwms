import React from 'react';
import { render, screen } from '@testing-library/react';
import { ProjectPageContainer } from './project-page-container.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

it('renders header (title, subTitle, extra) and children', () => {
  render(
    <ProjectPageContainer title="Project" subTitle="demo" extra={<button data-testid="page-action">Save</button>}>
      <div data-testid="page-children">content</div>
    </ProjectPageContainer>,
  );
  expect(screen.getByTestId('project-page-container')).toBeTruthy();
  expect(screen.getByText('Project')).toBeTruthy();
  expect(screen.getByText('demo')).toBeTruthy();
  expect(screen.getByTestId('page-action')).toBeTruthy();
  expect(screen.getByTestId('page-children')).toBeTruthy();
});

it('renders breadcrumb, tabs and footer', () => {
  render(
    <ProjectPageContainer
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
  const { rerender } = render(<ProjectPageContainer loading />);
  expect(screen.getByTestId('project-page-container').getAttribute('data-loading')).toBe('true');
  expect(screen.getByTestId('project-page-container-loading')).toBeTruthy();
  rerender(<ProjectPageContainer loading={false} />);
  expect(screen.getByTestId('project-page-container').getAttribute('data-loading')).toBe('false');
});
