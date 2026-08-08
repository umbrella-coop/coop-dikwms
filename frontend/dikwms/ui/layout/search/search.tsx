import type { ReactNode } from 'react';
import { Input } from 'antd';
import styles from './search.module.css';

export type LayoutSearchProps = {
  placeholder?: string;
  value?: string;
  onChange?: (value: string) => void;
  /**
   * Quick-action element on the right (ProLayout PlusCircleFilled parity).
   */
  actionIcon?: ReactNode;
  onActionClick?: () => void;
  'data-testid'?: string;
};

export function LayoutSearch({
  placeholder,
  value,
  onChange,
  actionIcon,
  onActionClick,
  'data-testid': testId = 'layout-search',
}: LayoutSearchProps) {
  return (
    <div className={styles.wrap} data-testid={testId}>
      <Input
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange?.(e.target.value)}
        variant="borderless"
        prefix={<span aria-hidden>🔍</span>}
        className={styles.input}
        data-testid={`${testId}-input`}
      />
      {actionIcon && (
        <span className={styles.action} onClick={onActionClick} data-testid={`${testId}-action`}>
          {actionIcon}
        </span>
      )}
    </div>
  );
}
