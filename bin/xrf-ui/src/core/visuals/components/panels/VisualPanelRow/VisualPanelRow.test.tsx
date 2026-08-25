import { describe, expect, it } from "@jest/globals";
import { Chip } from "@mui/material";

import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow/VisualPanelRow";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("VisualPanelRow", () => {
  it("shows a label and its value", () => {
    const { getByText } = renderWithProviders(<VisualPanelRow label={"Texture"} value={"wpn_ak74.dds"} />);

    expect(getByText("Texture")).toBeInTheDocument();
    expect(getByText("wpn_ak74.dds")).toBeInTheDocument();
  });

  it("holds an element value outside a paragraph", () => {
    const { container, getByText } = renderWithProviders(
      <VisualPanelRow label={"Texture"} value={<Chip size={"small"} label={"Loading"} />} />
    );

    expect(container.querySelector("p > .MuiChip-root")).toBeNull();
    expect(getByText("Loading")).toBeInTheDocument();
  });
});
