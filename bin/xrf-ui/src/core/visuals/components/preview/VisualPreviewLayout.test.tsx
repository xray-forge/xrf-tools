import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { VisualPreviewLayout } from "@/core/visuals/components/preview/VisualPreviewLayout";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

/**
 * Renders the layout over a stub viewport.
 *
 * @param footer - Footer the caller supplies, or none.
 * @returns The render result.
 */
function renderLayout(footer?: string): RenderResult {
  return renderWithProviders(
    <VisualPreviewLayout
      model={mockVisualModelViews()}
      footer={footer ? <div>{footer}</div> : undefined}
      renderViewport={() => <div data-testid={"stub-viewport"} />}
    />
  );
}

describe("VisualPreviewLayout footer", () => {
  it("draws nothing under the viewport unless the caller asks for it", () => {
    const { getByTestId, queryByRole } = renderLayout();

    expect(getByTestId("stub-viewport")).toBeInTheDocument();
    expect(queryByRole("button", { name: "Play" })).toBeNull();
    expect(queryByRole("combobox", { name: "Motion" })).toBeNull();
    expect(queryByRole("textbox", { name: "Filter motions" })).toBeNull();
  });

  it("draws the footer a surface poses, at whatever height it asks for", () => {
    const { getByText } = renderLayout("sequencer transport");

    expect(getByText("sequencer transport")).toBeInTheDocument();
  });
});
