/**
 * Data Graph primitives (DIKW Data layer) — shared wire-agnostic types.
 * Parity: backend `data-graph` EntityKind { Node, Edge, Combo }.
 * Schemaless: `kind` stays loose (string), like the wire.
 */

/**
 * Discriminating node shape (SPEC-027): the wire's generic `kind` ("Node")
 * cannot tell git entities apart, so the API resolves a namespace-qualified
 * shape from the registry (git.v1 messages) per entity. `GraphNodeShape` is
 * the frontend union of those shapes, mapped from the wire by `shapeFromWire`.
 */
export type GraphNodeShape =
  | 'node'
  | 'commit'
  | 'author'
  | 'repository'
  | 'organization'
  | 'insight';

const WIRE_SHAPES: Record<string, GraphNodeShape> = {
  'git.v1/Commit': 'commit',
  'git.v1/Author': 'author',
  'git.v1/Repository': 'repository',
  'git.v1/Organization': 'organization',
  'git.v1/Insight': 'insight',
};

/** Map a wire shape (e.g. "git.v1/Commit") to the frontend union; unknown or
 * absent shapes fall back to the generic "node". */
export function shapeFromWire(shape: string | undefined): GraphNodeShape {
  return (shape && WIRE_SHAPES[shape]) || 'node';
}

export type DataGraphNode = {
  id: string;
  kind: string;
  name?: string;
  /** Discriminating shape (SPEC-027): distinguishes git entities. */
  shape?: GraphNodeShape;
};

export type DataGraphEdge = {
  id: string;
  kind: string;
  source: string;
  target: string;
  name?: string;
};

export type DataGraphCombo = {
  id: string;
  kind: string;
  name?: string;
  members?: string[];
};
