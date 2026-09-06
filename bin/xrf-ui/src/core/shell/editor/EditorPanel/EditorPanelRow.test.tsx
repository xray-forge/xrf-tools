import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { EditorPanelRow } from "@/core/shell/editor/EditorPanel";
import { PANEL } from "@/core/theme/tokens";
import { renderWithProviders } from "@/fixtures/utils/render";

const REFERENCE: string = "aaz\\actor\\act_aaz_svoboda4";

function renderRow(value: string, isMonospace: boolean = false): HTMLElement {
  const render: RenderResult = renderWithProviders(
    <EditorPanelRow label={"Texture"} value={value} isMonospace={isMonospace} />
  );

  return render.getByTestId("visual-panel-row");
}

describe("EditorPanelRow", () => {
  it("shows the label and the value", () => {
    const row: HTMLElement = renderRow("DXT5");

    expect(row).toHaveTextContent("Texture");
    expect(row).toHaveTextContent("DXT5");
  });

  it("lets the value take a line of its own rather than shrink to nothing", () => {
    // The row is the only thing that knows how much room it was given: the panel is dragged between a couple of
    // hundred pixels and six hundred, and no caller can be asked to guess which shape it will need.
    const row: HTMLElement = renderRow(REFERENCE);
    const value: HTMLElement = row.querySelector("span") as HTMLElement;

    expect(row).toHaveStyle({ flexWrap: "wrap" });
    expect(value).toHaveStyle({ minWidth: PANEL.rowValueMinWidth });
  });

  it("offers a path its separators to break on", () => {
    // `overflow-wrap` alone splits a name wherever the line runs out, which is how `act_aaz_svoboda4` ends up across
    // two lines at no meaningful point. The separators come first, so a path folds where a reader would fold it.
    const row: HTMLElement = renderRow(REFERENCE, true);

    expect(row.querySelectorAll("wbr")).toHaveLength(3);
    expect(row).toHaveTextContent(REFERENCE);
  });

  it("leaves a value with nothing to break on alone", () => {
    const row: HTMLElement = renderRow("act_aaz_svoboda4", true);

    expect(row.querySelectorAll("wbr")).toHaveLength(0);
    expect(row).toHaveTextContent("act_aaz_svoboda4");
  });

  it("renders a value that is not text at all", () => {
    // Chips and badges go through here too, and they carry their own layout.
    const render: RenderResult = renderWithProviders(
      <EditorPanelRow label={"Bump"} value={<span data-testid={"chip"}>Flat</span>} isMonospace />
    );

    expect(render.getByTestId("chip")).toBeInTheDocument();
  });
});
