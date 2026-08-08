import type { ReactNode } from 'react';
import { Layout, Menu } from 'antd';
import type { MenuProps } from 'antd';
import styles from './frame.module.css';

export type LayoutFrameMenuItem = {
  key: string;
  label: ReactNode;
  icon?: ReactNode;
  children?: Omit<LayoutFrameMenuItem, 'children'>[];
};

export type LayoutFrameProps = {
  /**
   * Layout orientation ('side' | 'mix' | 'top'). ProLayout parity.
   */
  layout?: 'side' | 'mix' | 'top';
  /**
   * Fix the sider when the content scrolls.
   */
  fixSider?: boolean;
  /**
   * Menu tree (groups via `children` — siderMenuType 'group' parity).
   */
  menu?: LayoutFrameMenuItem[];
  /**
   * Selected menu key (driven by route pathname).
   */
  selectedKey?: string;
  onMenuSelect?: (key: string) => void;
  /**
   * Header left slot — pass <LayoutHeaderTitle/> (+ mega menu trigger).
   */
  headerTitle?: ReactNode;
  /**
   * Header right slot — pass <LayoutSearch/>, <LayoutHeaderActions/>, <LayoutUserMenu/>.
   */
  headerActions?: ReactNode;
  /**
   * Sider bottom slot — pass <LayoutMenuFooter/>.
   */
  siderFooter?: ReactNode;
  collapsed?: boolean;
  onCollapsedChange?: (collapsed: boolean) => void;
  children?: ReactNode;
  'data-testid'?: string;
};

const { Header, Sider, Content } = Layout;

export function LayoutFrame({
  layout = 'side',
  fixSider,
  menu = [],
  selectedKey,
  onMenuSelect,
  headerTitle,
  headerActions,
  siderFooter,
  collapsed,
  onCollapsedChange,
  children,
  'data-testid': testId = 'layout-frame',
}: LayoutFrameProps) {
  const items = toAntdItems(menu);
  const menuNode = (
    <Menu
      mode="inline"
      items={items}
      selectedKeys={selectedKey ? [selectedKey] : undefined}
      onClick={(e) => onMenuSelect?.(e.key)}
      data-testid={`${testId}-menu`}
    />
  );
  const headerNode = (
    <Header className={styles.header} data-testid={`${testId}-header`}>
      <div className={styles.headerLeft}>{headerTitle}</div>
      <div className={styles.headerRight}>{headerActions}</div>
    </Header>
  );
  const contentNode = <Content className={styles.content}>{children}</Content>;

  if (layout === 'top') {
    return (
      <section className={styles.wrap} data-testid={testId} data-layout={layout}>
        <Layout className={styles.fill}>
          {headerNode}
          <div className={styles.topMenu}>{menuNode}</div>
          {contentNode}
        </Layout>
      </section>
    );
  }
  return (
    <section className={styles.wrap} data-testid={testId} data-layout={layout} data-collapsed={collapsed ?? false}>
      <Layout className={styles.fill}>
        <Sider
          collapsible
          collapsed={collapsed}
          onCollapse={onCollapsedChange}
          theme="light"
          className={fixSider ? styles.siderFixed : styles.sider}
        >
          <div className={styles.siderBody}>{menuNode}</div>
          {siderFooter && <div className={styles.siderFooter}>{siderFooter}</div>}
        </Sider>
        <Layout>
          {headerNode}
          {contentNode}
        </Layout>
      </Layout>
    </section>
  );
}

function toAntdItems(menu: LayoutFrameMenuItem[]): MenuProps['items'] {
  return menu.map((m) => ({
    key: m.key,
    icon: m.icon,
    label: m.label,
    children: m.children?.map((c) => ({ key: c.key, icon: c.icon, label: c.label })),
  }));
}
