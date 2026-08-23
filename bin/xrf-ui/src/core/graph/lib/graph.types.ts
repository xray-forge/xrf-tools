import { Edge, Node } from "@xyflow/react";

/**
 * A node on any of this application's graphs.
 *
 * Aliased so surfaces name one type rather than importing the graph library directly, and so the
 * data payload stays the caller's business.
 */
export type TGraphNode<TData extends Record<string, unknown> = Record<string, unknown>> = Node<TData>;

export type TGraphEdge = Edge;

/** Rank direction a layout flows in. `TB` reads as a conversation, `LR` as a pipeline. */
export type TGraphDirection = "TB" | "LR";

export interface IGraphLayoutOptions {
  direction?: TGraphDirection;
  /** Stand-in size for a node the canvas has not measured yet. */
  nodeWidth?: number;
  nodeHeight?: number;
  /** Gap between ranks, along the flow direction. */
  rankSeparation?: number;
  /** Gap between nodes sharing a rank. */
  nodeSeparation?: number;
}

/**
 * Layout defaults.
 *
 * Sized for nodes that hold a line or two of text, which is what every graph surface here draws so
 * far. A caller whose nodes are larger passes its own measurements rather than nudging these.
 */
export const GRAPH_LAYOUT_DEFAULTS: Required<IGraphLayoutOptions> = {
  direction: "TB",
  nodeHeight: 80,
  nodeSeparation: 40,
  nodeWidth: 220,
  rankSeparation: 90,
};
