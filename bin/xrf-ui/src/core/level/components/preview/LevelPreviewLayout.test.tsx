import { describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent, RenderResult } from "@testing-library/react";

import { LevelPreviewLayout } from "@/core/level/components/preview/LevelPreviewLayout";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { ILevelStreamProgress } from "@/core/level/services";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { mockVisualBounds } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Maybe } from "@/lib/types/general";

const IDLE: ILevelStreamProgress = { loaded: 0, total: 0 };

/**
 * Renders the layout over a stub viewport, since a test has no webgl context to give it a real one.
 */
function renderLayout(overrides: Partial<Parameters<typeof LevelPreviewLayout>[0]> = {}): RenderResult {
  return renderWithProviders(
    <LevelPreviewLayout
      sectors={new Map()}
      bounds={mockVisualBounds()}
      name={"levels\\zaton"}
      streaming={IDLE}
      onCameraMoved={jest.fn()}
      renderViewport={() => <div data-testid={"stub-viewport"} />}
      {...overrides}
    />,
    { route: "/level-viewer" }
  );
}

describe("LevelPreviewLayout", () => {
  it("draws the level and names it", () => {
    const view: RenderResult = renderLayout();

    expect(view.getByTestId("stub-viewport")).toBeInTheDocument();
    expect(view.getByText("levels\\zaton")).toBeInTheDocument();
  });

  // Opening has nothing to look at yet, so it covers the viewport.
  it("reports opening a level over the viewport", () => {
    const view: RenderResult = renderLayout({ isLoading: true, name: null });

    expect(view.getByRole("status")).toHaveTextContent("Opening level");
    expect(view.queryByTestId("level-stream-progress")).not.toBeInTheDocument();
  });

  // Streaming does not: what has arrived is already drawn and already flyable, so the progress sits over it.
  it("reports streaming without taking the viewport away", () => {
    const view: RenderResult = renderLayout({ streaming: { loaded: 3, total: 24 } });

    expect(view.getByTestId("stub-viewport")).toBeInTheDocument();
    expect(view.getByTestId("level-stream-progress")).toHaveTextContent("Streaming sectors, 3 of 24");
  });

  it("shows no progress once nothing is in flight", () => {
    const view: RenderResult = renderLayout();

    expect(view.queryByTestId("level-stream-progress")).not.toBeInTheDocument();
  });

  it("says nothing is open when nothing is", () => {
    const view: RenderResult = renderLayout({ name: null });

    expect(view.getByText("No level open")).toBeInTheDocument();
  });

  it("reports why an open failed and offers it again", () => {
    const onRetry = jest.fn();
    const view: RenderResult = renderLayout({ error: "level carries no visuals chunk", name: null, onRetry });

    expect(view.getByText("level carries no visuals chunk")).toBeInTheDocument();

    fireEvent.click(view.getByRole("button", { name: "Retry" }));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("returns to the picker through the application breadcrumb", () => {
    const onBack = jest.fn();
    const view: RenderResult = renderLayout({ onBack });

    fireEvent.click(view.getByRole("button", { name: "Back to Level viewer" }));

    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("closes the open level from the file header", () => {
    const onDeselect = jest.fn();
    const view: RenderResult = renderLayout({ onDeselect });

    fireEvent.click(view.getByRole("button", { name: "Close level" }));

    expect(onDeselect).toHaveBeenCalledTimes(1);
  });

  // Flying a level is the whole interaction, and a level is a kilometre of ground that looks the same from most of
  // it: without a readout there is no way to say where you are, or to go back to where you were.
  it("says where the camera is, labelled by axis", async () => {
    let report: Maybe<(point: ILevelPoint) => void> = null;
    const view: RenderResult = renderWithProviders(
      <>
        <LevelPreviewLayout
          sectors={new Map()}
          bounds={mockVisualBounds()}
          name={"levels\\zaton"}
          streaming={IDLE}
          onCameraMoved={jest.fn()}
          renderViewport={({ onCameraChanged }) => {
            report = onCameraChanged;

            return <div data-testid={"stub-viewport"} />;
          }}
        />
        <ApplicationStatusBar />
      </>,
      { route: "/level-viewer" }
    );

    act(() => report?.({ x: -243.75, y: 12.5, z: 87.25 }));

    expect(await view.findByText("x -243.8 y 12.5 z 87.3")).toBeInTheDocument();
  });
});
