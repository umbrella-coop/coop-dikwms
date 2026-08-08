import React from 'react';
import { render, screen } from '@testing-library/react';
import { WorkspaceConsolePageIndexContainer } from './console-page-index-container.js';

it('renders header (title, description, extra) and children', () => {
  render(
    <WorkspaceConsolePageIndexContainer
      title="Workspace console"
      description="Manage projects in this workspace"
      extra={<button data-testid="ws-action">New project</button>}
    >
      <div data-testid="ws-children">content</div>
    </WorkspaceConsolePageIndexContainer>,
  );
  expect(screen.getByTestId('workspace-console-page-index-container')).toBeTruthy();
  expect(screen.getByText('Workspace console')).toBeTruthy();
  expect(screen.getByText('Manage projects in this workspace')).toBeTruthy();
  expect(screen.getByTestId('ws-action')).toBeTruthy();
  expect(screen.getByTestId('ws-children')).toBeTruthy();
});

it('reflects loading on the DOM', () => {
  const { rerender } = render(<WorkspaceConsolePageIndexContainer loading />);
  expect(screen.getByTestId('workspace-console-page-index-container').getAttribute('data-loading')).toBe('true');
  expect(screen.getByTestId('workspace-console-page-index-container-loading')).toBeTruthy();
  rerender(<WorkspaceConsolePageIndexContainer loading={false} />);
  expect(screen.getByTestId('workspace-console-page-index-container').getAttribute('data-loading')).toBe('false');
});
