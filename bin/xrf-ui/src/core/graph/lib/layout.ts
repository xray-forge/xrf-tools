import { EdgeLabel, Graph, GraphLabel, layout, NodeLabel } from "@dagrejs/dagre";

import { GRAPH_LAYOUT_DEFAULTS, IGraphLayoutOptions, TGraphEdge, TGraphNode } from "@/core/graph/lib/graph.types";

/**
 * Positions nodes as a layered graph, in one pass.
 *
 * Layered rather than force-directed, and synchronous rather than animated: a dialog or spawn graph
 * is a DAG whose useful reading is "what follows what", and a simulation that keeps running has no
 * point at which the caller can say the layout is done. Cycles are handled by dagre, which breaks
 * them internally, so a graph that loops back terminates like any other.
 *
 * @param nodes - Nodes to place. Their existing positions are ignored.
 * @param edges - Edges deciding the ranking. Edges naming an absent node are skipped.
 * @param options - Direction, separation, and the fallback size for unmeasured nodes.
 * @returns The same nodes, in the same order, carrying computed positions.
 */
export function layoutGraphNodes<TNode extends TGraphNode>(
  nodes: ReadonlyArray<TNode>,
  edges: ReadonlyArray<TGraphEdge>,
  options: IGraphLayoutOptions = {}
): Array<TNode> {
  const { direction, nodeHeight, nodeSeparation, nodeWidth, rankSeparation }: Required<IGraphLayoutOptions> = {
    ...GRAPH_LAYOUT_DEFAULTS,
    ...options,
  };

  if (!nodes.length) {
    return [];
  }

  const graph: Graph<GraphLabel, NodeLabel, EdgeLabel> = new Graph({ directed: true });

  graph.setGraph({ nodesep: nodeSeparation, rankdir: direction, ranksep: rankSeparation });
  graph.setDefaultEdgeLabel(() => ({}));

  for (const node of nodes) {
    // `measured` is what the canvas rendered; `width`/`height` is what the caller declared. Either
    // beats the fallback, because a rank sized from the fallback overlaps once a node is taller.
    graph.setNode(node.id, {
      height: node.measured?.height ?? node.height ?? nodeHeight,
      width: node.measured?.width ?? node.width ?? nodeWidth,
    });
  }

  // An edge naming a node nobody passed would make dagre invent one, and the invented node takes a
  // rank of its own and pushes every real node sideways.
  for (const edge of edges) {
    if (graph.hasNode(edge.source) && graph.hasNode(edge.target)) {
      graph.setEdge(edge.source, edge.target);
    }
  }

  layout(graph);

  return nodes.map((node: TNode) => {
    const placed: NodeLabel = graph.node(node.id);

    // Dagre reports a node's center; the canvas positions by its top-left corner.
    return {
      ...node,
      position: {
        x: (placed.x ?? 0) - placed.width / 2,
        y: (placed.y ?? 0) - placed.height / 2,
      },
    };
  });
}
