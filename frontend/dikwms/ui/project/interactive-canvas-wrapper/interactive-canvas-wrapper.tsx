import { Result } from 'antd';
import type { ReactNode } from 'react';
import { DataGraphProjectLiveViewContainer } from '@coop-codes/dikwms.ui.data-graph.project-live-view-container';
import type { DataGraphProjectLiveViewContainerProps } from '@coop-codes/dikwms.ui.data-graph.project-live-view-container';

/**
 * Live project view types. A dikwms project can be interacted with through
 * multiple live representations; each is implemented by its own context:
 *   - 'graph': graph interaction (data-graph/project-live-view-container)
 *   - 'tabular': tabular interaction (data-tabular — future)
 *   - 'geographic': geographic interaction (data-geographic — future)
 */
export type ProjectLiveViewType = 'graph' | 'tabular' | 'geographic';

export type ProjectLiveViewWraperProps = Omit<DataGraphProjectLiveViewContainerProps, 'data-testid'> & {
  viewType: ProjectLiveViewType;
  'data-testid'?: string;
};

export function ProjectLiveViewWraper({ viewType, children, ...containerProps }: ProjectLiveViewWraperProps) {
  if (viewType === 'graph') {
    return <DataGraphProjectLiveViewContainer {...containerProps}>{children}</DataGraphProjectLiveViewContainer>;
  }
  const placeholders: Record<'tabular' | 'geographic', { title: string; subTitle: string; testId: string }> = {
    tabular: {
      title: 'Tabular live view',
      subTitle: 'Not implemented yet — coming as data-tabular/project-live-view-container.',
      testId: 'interactive-canvas-wrapper-tabular-placeholder',
    },
    geographic: {
      title: 'Geographic live view',
      subTitle: 'Not implemented yet — coming as data-geographic/project-live-view-container.',
      testId: 'interactive-canvas-wrapper-geographic-placeholder',
    },
  };
  const placeholder = placeholders[viewType];
  return (
    <Result status="info" title={placeholder.title} subTitle={placeholder.subTitle} data-testid={placeholder.testId} />
  );
}
