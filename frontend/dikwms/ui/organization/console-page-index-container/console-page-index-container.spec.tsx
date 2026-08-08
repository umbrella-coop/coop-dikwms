import React from 'react';
import { render, screen } from '@testing-library/react';
import { OrganizationConsolePageIndexContainer } from './console-page-index-container.js';

it('renders header (title, description, extra) and children', () => {
  render(
    <OrganizationConsolePageIndexContainer
      title="Organization console"
      description="Manage projects and workspaces"
      extra={<button data-testid="org-action">New project</button>}
    >
      <div data-testid="org-children">content</div>
    </OrganizationConsolePageIndexContainer>,
  );
  expect(screen.getByTestId('organization-console-page-index-container')).toBeTruthy();
  expect(screen.getByText('Organization console')).toBeTruthy();
  expect(screen.getByText('Manage projects and workspaces')).toBeTruthy();
  expect(screen.getByTestId('org-action')).toBeTruthy();
  expect(screen.getByTestId('org-children')).toBeTruthy();
});

it('reflects loading on the DOM', () => {
  const { rerender } = render(<OrganizationConsolePageIndexContainer loading />);
  expect(screen.getByTestId('organization-console-page-index-container').getAttribute('data-loading')).toBe('true');
  expect(screen.getByTestId('organization-console-page-index-container-loading')).toBeTruthy();
  rerender(<OrganizationConsolePageIndexContainer loading={false} />);
  expect(screen.getByTestId('organization-console-page-index-container').getAttribute('data-loading')).toBe('false');
});
