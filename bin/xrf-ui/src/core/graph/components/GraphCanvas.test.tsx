import { describe, expect, it } from "@jest/globals";

import { GraphCanvas } from "@/core/graph/components/GraphCanvas";
import { TGraphEdge, TGraphNode } from "@/core/graph/lib/graph.types";
import { renderWithProviders } from "@/fixtures/utils/render";

const NODES: Array<TGraphNode> = [
  { data: { label: "first" }, id: "first", position: { x: 0, y: 0 } },
  { data: { label: "second" }, id: "second", position: { x: 0, y: 120 } },
];

const EDGES: Array<TGraphEdge> = [{ id: "first-second", source: "first", target: "second" }];

describe("GraphCanvas", () => {
  it("renders its nodes and edges", () => {
    const { getByTestId, getByText } = renderWithProviders(<GraphCanvas nodes={NODES} edges={EDGES} />);

    expect(getByTestId("graph-canvas")).toBeInTheDocument();
    expect(getByText("first")).toBeInTheDocument();
    expect(getByText("second")).toBeInTheDocument();
  });

  it("renders overlays passed as children inside the viewport", () => {
    const { getByTestId } = renderWithProviders(
      <GraphCanvas nodes={NODES} edges={EDGES}>
        <div data-testid={"legend"} />
      </GraphCanvas>
    );

    expect(getByTestId("legend")).toBeInTheDocument();
  });

  it("renders an empty graph without a node or an error", () => {
    const { getByTestId, queryByText } = renderWithProviders(<GraphCanvas nodes={[]} edges={[]} />);

    expect(getByTestId("graph-canvas")).toBeInTheDocument();
    expect(queryByText("first")).not.toBeInTheDocument();
  });
});
