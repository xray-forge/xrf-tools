import { describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";

import { LevelPreviewLayout } from "@/core/level/components/preview/LevelPreviewLayout";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { EMPTY_LEVEL_STATS } from "@/core/level/lib/stats/level-stats";
import {
  IDLE_LEVEL_STREAM,
  ILevelStreamProgress,
  LevelLoadService,
  LevelRenderService,
  LevelViewportService,
  LevelViewService,
} from "@/core/level/services";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** A camera placed where a readout has something to say about every axis. */
function camera(position: Partial<ILevelPoint> = {}): ILevelCamera {
  return {
    // A quarter turn, which reads as 90 degrees rather than as a signed radian.
    heading: Math.PI / 2,
    pitch: 0,
    position: { x: -243.75, y: 12.5, z: 87.25, ...position },
  };
}

/**
 * Renders the layout beside the real status bar, over a stub viewport.
 *
 * @param onRender - Called on every render of the stub, so a test can count what the telemetry redraws.
 * @returns The rendered tree, and the service a test reports through.
 */
function renderReporting(onRender: () => void = () => undefined): {
  loader: LevelLoadService;
  view: RenderResult;
  viewport: LevelViewportService;
} {
  const container: Container = mockContainer([
    LevelLoadService,
    LevelRenderService,
    LevelViewService,
    LevelViewportService,
  ]);

  const view: RenderResult = renderWithProviders(
    <>
      <LevelPreviewLayout
        name={"levels\\zaton"}
        renderViewport={() => {
          onRender();

          return <div data-testid={"stub-viewport"} />;
        }}
      />
      <ApplicationStatusBar />
    </>,
    { container, route: "/level-viewer" }
  );

  return { loader: container.get(LevelLoadService), view, viewport: container.get(LevelViewportService) };
}

function renderLayout(
  overrides: Partial<Parameters<typeof LevelPreviewLayout>[0]> = {},
  streaming: ILevelStreamProgress = IDLE_LEVEL_STREAM
): RenderResult {
  const container: Container = mockContainer([
    LevelLoadService,
    LevelRenderService,
    LevelViewService,
    LevelViewportService,
  ]);

  setStreaming(container.get(LevelLoadService), streaming);

  return renderWithProviders(
    <LevelPreviewLayout
      name={"levels\\zaton"}
      renderViewport={() => <div data-testid={"stub-viewport"} />}
      {...overrides}
    />,
    { container, route: "/level-viewer" }
  );
}

function setStreaming(loader: LevelLoadService, streaming: ILevelStreamProgress): void {
  runInAction(() => {
    loader.streaming = streaming;
  });
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

  // A level assembling in view is worse than a wait: until it has been drawn with everything it opens with, a cover
  // says what it is waiting for.
  it("covers a level being read until it has been drawn whole", () => {
    const view: RenderResult = renderLayout({}, { loaded: 3, total: 24 });

    expect(view.getByTestId("level-preview-cover")).toHaveClass("opacity-100");
    expect(view.getByTestId("level-preview-cover")).toHaveTextContent("Reading sectors, 3 of 24");
    expect(view.queryByTestId("level-stream-progress")).not.toBeInTheDocument();
  });

  // A renderer that failed never draws the level, so the cover stays, and says why rather than waiting on.
  it("keeps the level covered and says why when the renderer fails", () => {
    const container: Container = mockContainer([
      LevelLoadService,
      LevelRenderService,
      LevelViewService,
      LevelViewportService,
    ]);

    container.get(LevelViewportService).reveal();
    runInAction(() => {
      container.get(LevelRenderService).failure = "No WebGPU adapter";
    });

    const view: RenderResult = renderWithProviders(
      <LevelPreviewLayout name={"levels\\zaton"} renderViewport={() => <div data-testid={"stub-viewport"} />} />,
      { container, route: "/level-viewer" }
    );

    expect(view.getByTestId("level-preview-cover")).toHaveClass("opacity-100");
    expect(view.getByRole("alert")).toHaveTextContent("The renderer stoppedNo WebGPU adapter");
    expect(view.queryByRole("progressbar")).not.toBeInTheDocument();
  });

  // Once shown, streaming does not take it away: what has arrived is drawn and flyable, so the progress sits over it.
  it("reports streaming without taking the viewport away once the level is shown", () => {
    const container: Container = mockContainer([
      LevelLoadService,
      LevelRenderService,
      LevelViewService,
      LevelViewportService,
    ]);

    setStreaming(container.get(LevelLoadService), { loaded: 3, total: 24 });
    container.get(LevelViewportService).reveal();

    const view: RenderResult = renderWithProviders(
      <LevelPreviewLayout name={"levels\\zaton"} renderViewport={() => <div data-testid={"stub-viewport"} />} />,
      { container, route: "/level-viewer" }
    );

    expect(view.getByTestId("stub-viewport")).toBeInTheDocument();
    expect(view.getByTestId("level-preview-cover")).toHaveClass("opacity-0");
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
  it("says where the camera is and which way it faces", async () => {
    const { view, viewport } = renderReporting();

    act(() => viewport.report(EMPTY_LEVEL_STATS, camera()));

    expect(await view.findByText("x -243.8 y 12.5 z 87.3")).toBeInTheDocument();
    expect(view.getByText("h 90.0° p 0.0°")).toBeInTheDocument();
  });

  // A clean view of the level is the reason to turn them off, and half a clean view is no use, so one switch takes
  // both away.
  it("takes both readouts off the viewport together", () => {
    const { view, viewport } = renderReporting();

    act(() => viewport.report(EMPTY_LEVEL_STATS, camera()));

    expect(view.getByTestId("level-preview-metrics")).toBeInTheDocument();
    expect(view.getByTestId("level-preview-coordinates")).toBeInTheDocument();

    fireEvent.click(view.getByRole("button", { name: "Readout" }));

    expect(view.queryByTestId("level-preview-metrics")).not.toBeInTheDocument();
    expect(view.queryByTestId("level-preview-coordinates")).not.toBeInTheDocument();
  });

  // The readout arrives four times a second for as long as a level is open. Held in this layout it re-rendered the
  // toolbar, the viewport element and the panel registration with it, which is a lot of React for two lines of text.
  it("redraws nothing that draws the level when the viewport reports", async () => {
    let renders: number = 0;
    const { view, viewport } = renderReporting(() => {
      renders += 1;
    });

    await view.findByTestId("stub-viewport");

    const before: number = renders;

    for (let tick = 0; tick < 20; tick += 1) {
      act(() => viewport.report(EMPTY_LEVEL_STATS, camera({ x: tick })));
    }

    expect(await view.findByText("x 19.0 y 12.5 z 87.3")).toBeInTheDocument();
    expect(renders).toBe(before);
  });

  it("says what the backend and the webview hold in the status bar", async () => {
    const megabyte: number = 1024 * 1024;

    setMockInvokeResponses({
      ["plugin:system|get_memory_usage"]: {
        application: { committed: 410 * megabyte, workingSet: 300 * megabyte, privateWorkingSet: 260 * megabyte },
        webview: [
          {
            kind: "gpu",
            pid: 8,
            memory: { committed: 3667 * megabyte, workingSet: 2100 * megabyte, privateWorkingSet: 2048 * megabyte },
          },
        ],
      },
    });

    const { view } = renderReporting();

    expect(await view.findByText("Backend 260 MB · Webview 2 GB")).toBeInTheDocument();
  });

  // A sector lands many times a second while a level streams in, and only the progress and the status bar say so.
  it("redraws nothing that draws the level as sectors arrive", async () => {
    let renders: number = 0;
    const { loader, view } = renderReporting(() => {
      renders += 1;
    });

    await view.findByTestId("stub-viewport");

    const before: number = renders;

    for (let loaded = 0; loaded < 20; loaded += 1) {
      act(() => setStreaming(loader, { loaded, total: 24 }));
    }

    expect(await view.findByText("Streaming sector 20 of 24")).toBeInTheDocument();
    expect(renders).toBe(before);
  });
});
