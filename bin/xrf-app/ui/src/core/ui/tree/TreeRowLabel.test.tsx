import { describe, expect, it } from "@jest/globals";

import { ARCHIVED_CAPTION, TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("TreeRowLabel", () => {
  it("should render the name, which the tree shows nowhere else", () => {
    // `VirtualizedTree` renders a `renderLabel` result in place of the row's text rather than beside it, so a label
    // answering only a caption leaves a tree of captions with no names.
    const { getByText } = renderWithProviders(<TreeRowLabel label={"w_ak74.ltx"} caption={ARCHIVED_CAPTION} />);

    expect(getByText("w_ak74.ltx")).toBeInTheDocument();
    expect(getByText(ARCHIVED_CAPTION)).toBeInTheDocument();
  });

  it("should say nothing after a name it has nothing to add to", () => {
    const { getByText, queryByText } = renderWithProviders(<TreeRowLabel label={"w_ak74.ltx"} />);

    expect(getByText("w_ak74.ltx")).toBeInTheDocument();
    expect(queryByText(ARCHIVED_CAPTION)).not.toBeInTheDocument();
  });

  it("should spell the archive caption one way for every tree that says it", () => {
    // Three trees mark the same fact; a second spelling of it is the drift this constant exists to prevent.
    expect(ARCHIVED_CAPTION).toBe("db");
  });
});
