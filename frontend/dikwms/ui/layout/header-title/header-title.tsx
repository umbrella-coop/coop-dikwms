import type { ReactNode } from 'react';
import styles from './header-title.module.css';

export type LayoutHeaderTitleProps = {
  /**
   * Logo element (img src or ReactNode).
   */
  logo?: ReactNode;
  /**
   * Shell title text (e.g. "dikwms").
   */
  title?: ReactNode;
  /**
   * Extra slot on the right (e.g. <LayoutMegaMenu/> trigger).
   */
  extra?: ReactNode;
  onClick?: () => void;
  'data-testid'?: string;
};

export function LayoutHeaderTitle({
  logo,
  title,
  extra,
  onClick,
  'data-testid': testId = 'layout-header-title',
}: LayoutHeaderTitleProps) {
  return (
    <div className={styles.wrap} data-testid={testId} onClick={onClick} role={onClick ? 'button' : undefined}>
      {logo && <span className={styles.logo}>{typeof logo === 'string' ? <img src={logo} alt="" /> : logo}</span>}
      {title && <span className={styles.title}>{title}</span>}
      {extra}
    </div>
  );
}
