import type { ReactNode } from 'react';
import { Divider, Popover, Typography } from 'antd';
import styles from './mega-menu.module.css';

export type LayoutMegaMenuGroup = { title: string; items: string[] };
export type LayoutMegaMenuHot = { name: string; desc: string };

export type LayoutMegaMenuProps = {
  /**
   * Category columns (ProLayout MenuCard "解决方案" parity).
   */
  groups?: LayoutMegaMenuGroup[];
  /**
   * Hot products column ("热门产品" parity).
   */
  hot?: LayoutMegaMenuHot[];
  /**
   * Trigger element (usually inside <LayoutHeaderTitle/> extra).
   */
  trigger: ReactNode;
  'data-testid'?: string;
};

export function LayoutMegaMenu({
  groups = [],
  hot = [],
  trigger,
  'data-testid': testId = 'layout-mega-menu',
}: LayoutMegaMenuProps) {
  return (
    <Popover
      placement="bottom"
      trigger="click"
      data-testid={testId}
      content={
        <div className={styles.content} data-testid={`${testId}-content`}>
          <div className={styles.categories}>
            {groups.map((g) => (
              <div key={g.title} className={styles.category}>
                <div className={styles.categoryTitle}>{g.title}</div>
                {g.items.map((item) => (
                  <div key={item} className={styles.categoryItem}>
                    {item}
                  </div>
                ))}
              </div>
            ))}
          </div>
          {hot.length > 0 && (
            <>
              <Divider orientation="vertical" className={styles.divider} />
              <div className={styles.hot}>
                <div className={styles.hotTitle}>热门产品</div>
                {hot.map((p) => (
                  <div key={p.name} className={styles.hotItem}>
                    <div className={styles.hotName}>{p.name}</div>
                    <div className={styles.hotDesc}>{p.desc}</div>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      }
    >
      {trigger}
    </Popover>
  );
}
