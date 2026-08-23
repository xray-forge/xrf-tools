import { describe, expect, it } from "@jest/globals";

import { GRAPH_LAYOUT_DEFAULTS, TGraphEdge, TGraphNode } from "@/core/graph/lib/graph.types";
import { layoutGraphNodes } from "@/core/graph/lib/layout";

/**
 * Builds a node with no position, which is what the layout is for.
 *
 * @param id - Node identity.
 * @param measured - Rendered size, when the canvas has reported one.
 * @param measured.width - Rendered width.
 * @param measured.height - Rendered height.
 * @returns A node the layout can place.
 */
function createNode(id: string, measured?: { width: number; height: number }): TGraphNode {
  return { data: {}, id, measured, position: { x: 0, y: 0 } };
}

/**
 * Builds an edge between two node ids.
 *
 * @param source - Edge origin.
 * @param target - Edge destination.
 * @returns An edge the layout can rank by.
 */
function createEdge(source: string, target: string): TGraphEdge {
  return { id: `${source}-${target}`, source, target };
}

describe("layoutGraphNodes", () => {
  it("returns nothing for an empty graph", () => {
    expect(layoutGraphNodes([], [])).toEqual([]);
  });

  it("keeps the input order and identity", () => {
    const placed: Array<TGraphNode> = layoutGraphNodes(
      [createNode("c"), createNode("a"), createNode("b")],
      [createEdge("a", "b")]
    );

    expect(placed.map((node: TGraphNode) => node.id)).toEqual(["c", "a", "b"]);
  });

  it("ranks a target below its source when flowing top to bottom", () => {
    const [root, child]: Array<TGraphNode> = layoutGraphNodes(
      [createNode("root"), createNode("child")],
      [createEdge("root", "child")]
    );

    expect(child.position.y).toBeGreaterThan(root.position.y);
    expect(child.position.y - root.position.y).toBeGreaterThanOrEqual(GRAPH_LAYOUT_DEFAULTS.rankSeparation);
  });

  it("ranks a target right of its source when flowing left to right", () => {
    const [root, child]: Array<TGraphNode> = layoutGraphNodes(
      [createNode("root"), createNode("child")],
      [createEdge("root", "child")],
      { direction: "LR" }
    );

    expect(child.position.x).toBeGreaterThan(root.position.x);
  });

  it("positions by the top-left corner rather than the center dagre reports", () => {
    const [only]: Array<TGraphNode> = layoutGraphNodes([createNode("only", { width: 200, height: 100 })], []);

    // A lone node is centered in its own graph, so its corner sits at half its size from the origin.
    expect(only.position).toEqual({ x: 0, y: 0 });
  });

  it("separates siblings by their measured width, not by the fallback", () => {
    const [, first, second]: Array<TGraphNode> = layoutGraphNodes(
      [
        createNode("root", { width: 100, height: 40 }),
        createNode("first", { width: 400, height: 40 }),
        createNode("second", { width: 400, height: 40 }),
      ],
      [createEdge("root", "first"), createEdge("root", "second")]
    );

    // Distance, not order: which sibling dagre puts on the left is its own business, but they must
    // not overlap, which they would if both were sized from the 220px fallback.
    expect(Math.abs(second.position.x - first.position.x)).toBeGreaterThanOrEqual(400);
  });

  it("is deterministic, so reopening a graph does not move it", () => {
    const nodes: Array<TGraphNode> = [createNode("a"), createNode("b"), createNode("c")];
    const edges: Array<TGraphEdge> = [createEdge("a", "b"), createEdge("a", "c"), createEdge("b", "c")];

    expect(layoutGraphNodes(nodes, edges)).toEqual(layoutGraphNodes(nodes, edges));
  });

  it("terminates on a cycle", () => {
    // The force-directed layouter this replaces hung on exactly this shape, and dialogs loop back to
    // their root constantly.
    const placed: Array<TGraphNode> = layoutGraphNodes(
      [createNode("a"), createNode("b"), createNode("c")],
      [createEdge("a", "b"), createEdge("b", "c"), createEdge("c", "a")]
    );

    expect(placed).toHaveLength(3);
    expect(placed.every((node: TGraphNode) => Number.isFinite(node.position.x))).toBe(true);
  });

  it("places a disconnected node without throwing", () => {
    const placed: Array<TGraphNode> = layoutGraphNodes(
      [createNode("a"), createNode("b"), createNode("orphan")],
      [createEdge("a", "b")]
    );

    expect(placed).toHaveLength(3);
    expect(placed.every((node: TGraphNode) => Number.isFinite(node.position.y))).toBe(true);
  });

  it("skips an edge naming a node it was not given", () => {
    // Left in, dagre invents the missing node and gives it a rank, which shifts every real node.
    const withDangling: Array<TGraphNode> = layoutGraphNodes(
      [createNode("a"), createNode("b")],
      [createEdge("a", "b"), createEdge("b", "missing")]
    );
    const withoutDangling: Array<TGraphNode> = layoutGraphNodes(
      [createNode("a"), createNode("b")],
      [createEdge("a", "b")]
    );

    expect(withDangling).toEqual(withoutDangling);
  });
});
