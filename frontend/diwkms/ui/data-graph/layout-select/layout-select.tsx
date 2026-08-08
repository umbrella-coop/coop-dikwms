import { Select } from 'antd';

export const LAYOUT_OPTIONS: { value: string; label: string }[] = [
  { value: 'force', label: 'Force (default)' },
  { value: 'd3-force', label: 'D3 Force' },
  { value: 'fruchterman', label: 'Fruchterman' },
  { value: 'grid', label: 'Grid' },
  { value: 'circular', label: 'Circular' },
  { value: 'radial', label: 'Radial' },
  { value: 'concentric', label: 'Concentric' },
  { value: 'mds', label: 'MDS' },
  { value: 'random', label: 'Random' },
];

export type LayoutSelectProps = {
  value: string;
  onChange: (value: string) => void;
};

export function LayoutSelect({ value, onChange }: LayoutSelectProps) {
  return (
    <Select
      value={value}
      onChange={onChange}
      options={LAYOUT_OPTIONS}
      size="small"
      data-testid="layout-select"
      aria-label="Canvas layout type"
    />
  );
}
