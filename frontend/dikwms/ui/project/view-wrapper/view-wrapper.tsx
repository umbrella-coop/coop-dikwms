import { Result } from 'antd';
import type { ReactNode } from 'react';
import { DataGraphProjectLiveViewContainer } from '@coop-codes/dikwms.ui.data-graph.project-live-view-container';
import type { DataGraphProjectLiveViewContainerProps } from '@coop-codes/dikwms.ui.data-graph.project-live-view-container';

/**
 * Project view types. A dikwms project can be viewed in multiple
 * representations; each is implemented by its own context:
 *   - 'live': graph live view (data-graph/project-live-view-container)
 *   - 'tabular': tabular representation (data-tabular — future)
 */
export type ProjectViewType = 'live' | 'tabular';

export type ProjectPageWraperProps = Omit<DataGraphProjectLiveViewContainerProps, 'data-testid'> & {
  viewType: ProjectViewType;
  'data-testid'?: string;
};

export function ProjectPageWraper({ viewType, children, ...containerProps }: ProjectPageWraperProps) {
  if (viewType === 'live') {
    return <DataGraphProjectLiveViewContainer {...containerProps}>{children}</DataGraphProjectLiveViewContainer>;
  }
  return (
    <Result
      status="info"
      title="Tabular view"
      subTitle="The tabular representation is not implemented yet — coming as data-tabular/project-view-container."
      data-testid="project-view-wrapper-tabular-placeholder"
    />
  );
}
