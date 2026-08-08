import { Avatar, Dropdown } from 'antd';
import type { MenuProps } from 'antd';

export type LayoutUserMenuItem = { key: string; label: string; icon?: React.ReactNode };

export type LayoutUserMenuProps = {
  /**
   * Avatar source / title (ProLayout avatarProps parity).
   */
  avatarSrc?: string;
  title?: string;
  items: LayoutUserMenuItem[];
  onSelect?: (key: string) => void;
  'data-testid'?: string;
};

export function LayoutUserMenu({
  avatarSrc,
  title,
  items,
  onSelect,
  'data-testid': testId = 'layout-user-menu',
}: LayoutUserMenuProps) {
  const menu: MenuProps = {
    items: items.map((i) => ({ key: i.key, label: i.label, icon: i.icon })),
    onClick: (e) => onSelect?.(e.key),
  };
  return (
    <Dropdown menu={menu} trigger={['click']} data-testid={testId}>
      <span className="layout-user-menu-trigger" data-testid={testId}>
        {avatarSrc ? <Avatar src={avatarSrc} size="small" /> : <Avatar size="small">{title?.[0] ?? 'U'}</Avatar>}
        {title && <span style={{ marginInlineStart: 8 }}>{title}</span>}
      </span>
    </Dropdown>
  );
}
