import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { Binding } from "@wirestate/core";

import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

let VisualPreviewLayout: typeof import("./VisualPreviewLayout").VisualPreviewLayout;

const source: IVisualRenderSource = { bumps: new Map(), model: mockVisualModelViews(), textures: new Map() };

const BINDINGS: Array<Binding> = [VisualViewService, { factory: () => source, token: VISUAL_RENDER_SOURCE }];

beforeAll(async () => {
  // Only the chrome around the viewport is under test, and the viewport itself cannot draw in jsdom.
  jest.doMock("@/core/visuals/components/preview/VisualPreviewViewport", () => ({
    VisualPreviewViewport: () => <div data-testid={"stub-viewport"} />,
  }));

  ({ VisualPreviewLayout } = await import("./VisualPreviewLayout"));
});

/**
 * Renders the layout over a stub viewport.
 *
 * @param footer - Footer the caller supplies, or none.
 * @returns The render result.
 */
function renderLayout(footer?: string): RenderResult {
  return renderWithProviders(<VisualPreviewLayout footer={footer ? <div>{footer}</div> : undefined} />, {
    bindings: BINDINGS,
  });
}

describe("VisualPreviewLayout footer", () => {
  it("returns to the picker through the application breadcrumb", () => {
    const onBack = jest.fn();
    const view = renderWithProviders(<VisualPreviewLayout onBack={onBack} />, {
      bindings: BINDINGS,
      route: "/visuals-explorer",
    });

    fireEvent.click(view.getByRole("button", { name: "Back to Visuals explorer" }));

    expect(onBack).toHaveBeenCalledTimes(1);
    expect(view.queryByRole("button", { name: "Open visual" })).not.toBeInTheDocument();
  });

  it("renders the session's location through the shared title-bar component", () => {
    const location = { path: "C:\\game\\database\\meshes.db" };
    const view = renderWithProviders(<VisualPreviewLayout subtitle={<EditorToolbarLocation location={location} />} />, {
      bindings: BINDINGS,
    });

    expect(view.getByTestId("editor-toolbar-location")).toHaveTextContent(location.path);
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
