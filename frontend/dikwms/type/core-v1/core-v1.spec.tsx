import type { DataGraphNode, DataGraphEdge, DataGraphCombo } from './core-v1.js';

it('shapes the graph primitives for consumers', () => {
  const node: DataGraphNode = { id: 'a', kind: 'Node' };
  const edge: DataGraphEdge = { id: 'e', kind: 'Edge', source: 'a', target: 'b' };
  const combo: DataGraphCombo = { id: 'c', kind: 'Combo', members: ['a', 'e'] };
  expect([node, edge, combo].length).toBe(3);
  expect(node.name).toBeUndefined();
});
