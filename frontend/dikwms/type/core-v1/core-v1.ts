/**
 * Data Graph primitives (DIKW Data layer) — shared wire-agnostic types.
 * Parity: backend `knowledge-domain` EntityKind { Node, Edge, Combo }.
 * Schemaless: `kind` stays loose (string), like the wire.
 */
export type DataGraphNode = { id: string; kind: string; name?: string };

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
