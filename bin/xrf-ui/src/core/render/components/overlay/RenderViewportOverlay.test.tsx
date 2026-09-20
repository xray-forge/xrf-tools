import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { RenderViewportOverlay, TRenderOverlayCorner } from "@/core/render/components/overlay";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderOverlay(corner: TRenderOverlayCorner = "top-left"): RenderResult {
  return renderWithProviders(<RenderViewportOverlay corner={corner}>31 fps</RenderViewportOverlay>);
}

describe("RenderViewportOverlay", () => {
  it("draws what it was given", () => {
    expect(renderOverlay().getByText("31 fps")).toBeInTheDocument();
  });

  // The whole point: a readout laid over a scene that answered a pointer would eat the drag that flies the camera,
  // and one that answered a selection would highlight its own numbers on every drag across it.
  it("is inert", () => {
    const overlay: HTMLElement = renderOverlay().getByTestId("render-viewport-overlay");

    expect(overlay.className).toContain("pointer-events-none");
    expect(overlay.className).toContain("select-none");
  });

  // The same facts reach the status bar and the panels, where a screen reader gets them in a form that is not a
  // floating scrap of text over a canvas.
  it("is not announced, since the readable copy lives elsewhere", () => {
    expect(renderOverlay().getByTestId("render-viewport-overlay")).toHaveAttribute("aria-hidden", "true");
  });

  it("sits in the corner it was asked for", () => {
    // One render each, because two overlays in one document answer the same test id.
    expect(renderOverlay("bottom-right").getByTestId("render-viewport-overlay").className).toContain(
      "bottom-2 right-2"
    );
    expect(renderOverlay("top-left").getAllByTestId("render-viewport-overlay").pop()?.className).toContain(
      "top-2 left-2"
    );
  });
});
