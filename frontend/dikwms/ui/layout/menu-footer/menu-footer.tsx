import type { ReactNode } from 'react';
import styles from './menu-footer.module.css';

export type LayoutMenuFooterProps = {
  /**
   * Footer lines (ProLayout menuFooterRender parity, e.g. copyright).
   */
  lines?: ReactNode[];
  'data-testid'?: string;
};

export function LayoutMenuFooter({ lines = [], 'data-testid': testId = 'layout-menu-footer' }: LayoutMenuFooterProps) {
  return (
    <div className={styles.wrap} data-testid={testId}>
      {lines.map((line, i) => (
        <div key={i}>{line}</div>
      ))}
    </div>
  );
}
