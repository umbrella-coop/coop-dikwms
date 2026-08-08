import type { ReactNode } from 'react';
import { Space, Spin, Typography } from 'antd';
import styles from './console-page-index-container.module.css';

export type ProjectConsolePageIndexContainerProps = {
  /**
   * Index page title (e.g. "Project console").
   */
  title?: ReactNode;
  /**
   * Short description under the title.
   */
  description?: ReactNode;
  /**
   * Actions rendered on the right of the header.
   */
  extra?: ReactNode;
  /**
   * Shows a spinner over the content area.
   */
  loading?: boolean;
  children?: ReactNode;
  'data-testid'?: string;
};

export function ProjectConsolePageIndexContainer({
  title,
  description,
  extra,
  loading,
  children,
  'data-testid': testId = 'project-console-page-index-container',
}: ProjectConsolePageIndexContainerProps) {
  return (
    <section
      className={styles.wrap}
      data-testid={testId}
      data-loading={loading ?? false}
    >
      <header className={styles.header}>
        <Space align="baseline" className={styles.titleRow}>
          <Typography.Title level={4} className={styles.title}>
            {title}
          </Typography.Title>
        </Space>
        {description && <p className={styles.description}>{description}</p>}
        {extra && <div className={styles.extra}>{extra}</div>}
      </header>
      <main className={styles.content}>
        {loading ? <Spin data-testid={`${testId}-loading`} /> : children}
      </main>
    </section>
  );
}
