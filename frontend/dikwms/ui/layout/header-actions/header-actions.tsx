import type { ReactNode } from 'react';
import { Tooltip } from 'antd';
import styles from './header-actions.module.css';

export type LayoutHeaderAction = {
  key: string;
  icon: ReactNode;
  tooltip?: string;
  onClick?: () => void;
};

export type LayoutHeaderActionsProps = {
  /**
   * Header icon actions (ProLayout actionsRender parity: Info/Question/Github...).
   */
  items: LayoutHeaderAction[];
  'data-testid'?: string;
};

export function LayoutHeaderActions({ items, 'data-testid': testId = 'layout-header-actions' }: LayoutHeaderActionsProps) {
  return (
    <div className={styles.wrap} data-testid={testId}>
      {items.map((item) => (
        <Tooltip key={item.key} title={item.tooltip}>
          <span
            className={styles.action}
            onClick={item.onClick}
            role={item.onClick ? 'button' : undefined}
            data-testid={`${testId}-${item.key}`}
          >
            {item.icon}
          </span>
        </Tooltip>
      ))}
    </div>
  );
}
