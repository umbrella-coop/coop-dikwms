import { shapeFromWire, type DataGraphNode, type DataGraphEdge, type DataGraphCombo } from './core-v1.js';

it('shapes the graph primitives for consumers', () => {
  const node: DataGraphNode = { id: 'a', kind: 'Node' };
  const edge: DataGraphEdge = { id: 'e', kind: 'Edge', source: 'a', target: 'b' };
  const combo: DataGraphCombo = { id: 'c', kind: 'Combo', members: ['a', 'e'] };
  expect([node, edge, combo].length).toBe(3);
  expect(node.name).toBeUndefined();
  expect(node.shape).toBeUndefined();
});

it('maps git.v1 wire shapes to the GraphNodeShape union (SPEC-027)', () => {
  expect(shapeFromWire('git.v1/Commit')).toBe('commit');
  expect(shapeFromWire('git.v1/Author')).toBe('author');
  expect(shapeFromWire('git.v1/Repository')).toBe('repository');
  expect(shapeFromWire('git.v1/Organization')).toBe('organization');
  expect(shapeFromWire('git.v1/Insight')).toBe('insight');
  expect(shapeFromWire(undefined)).toBe('node');
  expect(shapeFromWire('whatever/Unknown')).toBe('node');
});
