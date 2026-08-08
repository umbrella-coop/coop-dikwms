import React from 'react';
import { fireEvent, render, screen } from '@testing-library/react';
import { LayoutMegaMenu } from './mega-menu.js';

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver ??= ResizeObserverStub as unknown as typeof ResizeObserver;

it('opens the mega menu with categories and hot products', () => {
  render(
    <LayoutMegaMenu
      groups={[{ title: '通用方案', items: ['统一权限中心', '数据可视化引擎'] }]}
      hot={[{ name: '数据可视化引擎', desc: '企业级数据看板' }]}
      trigger={<span data-testid="mega-trigger">方案</span>}
    />,
  );
  fireEvent.click(screen.getByTestId('mega-trigger'));
  expect(screen.getByText('通用方案')).toBeTruthy();
  expect(screen.getByText('统一权限中心')).toBeTruthy();
  expect(screen.getAllByText('数据可视化引擎').length).toBeGreaterThanOrEqual(2);
  expect(screen.getByText('热门产品')).toBeTruthy();
  expect(screen.getByText('企业级数据看板')).toBeTruthy();
});
