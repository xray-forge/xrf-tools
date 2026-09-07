import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
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
  it("returns to the picker through the application breadcrumb", () => {
    const onBack = jest.fn();
    const view = renderWithProviders(
      <VisualPreviewLayout model={mockVisualModelViews()} onBack={onBack} renderViewport={() => <div />} />,
      { route: "/visuals-explorer" }
    );

    fireEvent.click(view.getByRole("button", { name: "Back to Visuals explorer" }));

    expect(onBack).toHaveBeenCalledTimes(1);
    expect(view.queryByRole("button", { name: "Open visual" })).not.toBeInTheDocument();
  });

  it("renders the full source location through the shared title-bar component", () => {
    const location = { path: "C:\\game\\database\\meshes.db", entry: "actors\\stalker.ogf" };
    const view = renderWithProviders(
      <VisualPreviewLayout
        model={mockVisualModelViews()}
        subtitle={<EditorToolbarLocation location={location} />}
        renderViewport={() => <div />}
      />
    );

    expect(view.getByTestId("editor-toolbar-location")).toHaveTextContent(location.path);
    expect(view.getByTestId("editor-toolbar-location")).toHaveTextContent(location.entry);
  });

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
