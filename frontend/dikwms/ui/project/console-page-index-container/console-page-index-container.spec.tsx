import React from 'react';
import { render, screen } from '@testing-library/react';
import { ProjectConsolePageIndexContainer } from './console-page-index-container.js';

it('renders header (title, description, extra) and children', () => {
  render(
    <ProjectConsolePageIndexContainer
      title="Project console"
      description="Manage the project views and data"
      extra={<button data-testid="prj-action">Open live view</button>}
    >
      <div data-testid="prj-children">content</div>
    </ProjectConsolePageIndexContainer>,
  );
  expect(screen.getByTestId('project-console-page-index-container')).toBeTruthy();
  expect(screen.getByText('Project console')).toBeTruthy();
  expect(screen.getByText('Manage the project views and data')).toBeTruthy();
  expect(screen.getByTestId('prj-action')).toBeTruthy();
  expect(screen.getByTestId('prj-children')).toBeTruthy();
});

it('reflects loading on the DOM', () => {
  const { rerender } = render(<ProjectConsolePageIndexContainer loading />);
  expect(screen.getByTestId('project-console-page-index-container').getAttribute('data-loading')).toBe('true');
  expect(screen.getByTestId('project-console-page-index-container-loading')).toBeTruthy();
  rerender(<ProjectConsolePageIndexContainer loading={false} />);
  expect(screen.getByTestId('project-console-page-index-container').getAttribute('data-loading')).toBe('false');
});
