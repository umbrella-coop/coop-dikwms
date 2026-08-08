import type { ReactNode } from 'react';
import { Breadcrumb, Space, Spin, Tabs, Typography } from 'antd';
import type { BreadcrumbProps, TabsProps } from 'antd';
import styles from './project-live-view-container.module.css';

export type DataGraphProjectLiveViewContainerProps = {
  /**
   * Page title, rendered in the header.
   */
  title?: ReactNode;
  /**
   * Small text next to the title.
   */
  subTitle?: ReactNode;
  /**
   * Actions rendered on the right of the header (Pro PageContainer `extra`).
   */
  extra?: ReactNode;
  /**
   * Breadcrumb items (Pro PageContainer `breadcrumb`).
   */
  breadcrumb?: BreadcrumbProps['items'];
  /**
   * Tab bar between header and content (Pro PageContainer `tabs`).
   */
  tabs?: TabsProps;
  /**
   * Static content above children (Pro PageContainer `content`).
   */
  content?: ReactNode;
  /**
   * Pinned footer area below children (Pro PageContainer `footer`).
   */
  footer?: ReactNode;
  /**
   * Transparent background variant.
   */
  ghost?: boolean;
  /**
   * Shows a spinner over the content area.
   */
  loading?: boolean;
  children?: ReactNode;
  'data-testid'?: string;
};

export function DataGraphProjectLiveViewContainer({
  title,
  subTitle,
  extra,
  breadcrumb,
  tabs,
  content,
  footer,
  ghost,
  loading,
  children,
  'data-testid': testId = 'project-live-view-container',
}: DataGraphProjectLiveViewContainerProps) {
  return (
    <section
      className={styles.wrap}
      data-testid={testId}
      data-loading={loading ?? false}
      data-ghost={ghost ?? false}
    >
      <header className={styles.header}>
        {breadcrumb && <Breadcrumb items={breadcrumb} />}
        <Space align="baseline" className={styles.titleRow}>
          <Typography.Title level={4} className={styles.title}>
            {title}
          </Typography.Title>
          {subTitle && <span className={styles.subTitle}>{subTitle}</span>}
        </Space>
        {extra && <div className={styles.extra}>{extra}</div>}
      </header>
      {tabs && <Tabs {...tabs} />}
      <main className={styles.content}>
        {content}
        {loading ? <Spin data-testid={`${testId}-loading`} /> : children}
      </main>
      {footer && <footer className={styles.footer}>{footer}</footer>}
    </section>
  );
}
