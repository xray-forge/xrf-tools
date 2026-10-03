import { describe, expect, it, jest } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";
import { LevelPreviewLayout } from "@/core/level/components/preview/LevelPreviewLayout";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelPoint } from "@/core/level/lib/camera/level-point";
import { EMPTY_LEVEL_SPAWN_REPORT } from "@/core/level/lib/spawn";
import {
  LevelLoadService,
  LevelLookService,
  LevelRenderService,
  LevelViewportService,
  LevelViewService,
  LevelWeatherService,
} from "@/core/level/services";
import { EMPTY_RENDER_FRAME_REPORT } from "@/core/render/lib/native/native-frame-report";
import { SettingsRendererDisplay } from "@/core/settings/components/SettingsDialog/SettingsRenderSection/SettingsRendererDisplay";
import { SettingsService } from "@/core/settings/services/settings";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

function camera(position: Partial<ILevelPoint> = {}): ILevelCamera {
  return {
    // A quarter turn, which reads as 90 degrees rather than as a signed radian.
    heading: Math.PI / 2,
    pitch: 0,
    position: { x: -243.75, y: 12.5, z: 87.25, ...position },
  };
}

function renderReporting(onRender: () => void = () => undefined): {
  loader: LevelLoadService;
  renderer: LevelRenderService;
  view: RenderResult;
  viewport: LevelViewportService;
} {
  const container: Container = mockContainer([
    LevelLoadService,
    LevelRenderService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
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

  return {
    loader: container.get(LevelLoadService),
    renderer: container.get(LevelRenderService),
    view,
    viewport: container.get(LevelViewportService),
  };
}

function renderLayout(
  overrides: Partial<Parameters<typeof LevelPreviewLayout>[0]> = {},
  load: Nullable<RenderLoadReport> = null
): RenderResult {
  const container: Container = mockContainer([
    LevelLoadService,
    LevelRenderService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
  ]);

  container.get(LevelViewportService).noteLoad(load);

  return renderWithProviders(
    <LevelPreviewLayout
      name={"levels\\zaton"}
      renderViewport={() => <div data-testid={"stub-viewport"} />}
      {...overrides}
    />,
    { container, route: "/level-viewer" }
  );
}

/** How far the renderer has read a level of 24 sectors and 40 textures: sectors first, then the textures. */
function toLoad(sectors: number, textures: number = 0): RenderLoadReport {
  return { bytes: 0, isReady: false, sectors, sectorsTotal: 24, textures, texturesTotal: 40 };
}

/** Opens the overlays, the readouts among them, and answers the popover. */
async function openOverlays(view: RenderResult): Promise<HTMLElement> {
  await userEvent.click(view.getByRole("button", { name: "Overlays" }));

  return view.findByRole("dialog", { name: "Overlays" });
}

async function closeOverlays(view: RenderResult): Promise<void> {
  await userEvent.keyboard("{Escape}");
  await waitFor(() => expect(view.queryByRole("dialog", { name: "Overlays" })).not.toBeInTheDocument());
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
  });

  // Coming in with the open, the header would shrink the viewport and have the renderer size its targets again.
  it("heads the viewport with the level being opened, and holds nothing open for it", () => {
    const view: RenderResult = renderLayout({ isLoading: true, name: null, pending: "levels\\zaton" });

    expect(view.getByTestId("level-file-header")).toHaveTextContent("levels\\zaton");
    expect(view.getByRole("status")).toHaveTextContent("Opening level");
    expect(view.queryByTestId("level-preview-pick")).not.toBeInTheDocument();
  });

  // A level assembling in view is worse than a wait: until it has been drawn with everything it opens with, a cover
  // says what it is waiting for.
  it("covers a level being read until it has been drawn whole", () => {
    const view: RenderResult = renderLayout({}, toLoad(3));

    expect(view.getByTestId("level-preview-cover")).toHaveClass("opacity-100");
    expect(view.getByTestId("level-preview-cover")).toHaveTextContent("Reading sectors, 3 of 24");
  });

  it("says it uploads the textures once every sector is read", () => {
    const view: RenderResult = renderLayout({}, toLoad(24, 10));

    expect(view.getByTestId("level-preview-cover")).toHaveTextContent("Uploading textures, 10 of 40");
  });

  // A renderer that failed never draws the level, so the cover stays, and says why rather than waiting on.
  it("keeps the level covered and says why when the renderer fails", () => {
    const container: Container = mockContainer([
      LevelLoadService,
      LevelRenderService,
      LevelViewService,
      LevelViewportService,
      LevelWeatherService,
      LevelLookService,
    ]);

    container.get(LevelViewportService).reveal();
    runInAction(() => {
      container.get(LevelRenderService).failure = "No GPU adapter";
    });

    const view: RenderResult = renderWithProviders(
      <LevelPreviewLayout name={"levels\\zaton"} renderViewport={() => <div data-testid={"stub-viewport"} />} />,
      { container, route: "/level-viewer" }
    );

    expect(view.getByTestId("level-preview-cover")).toHaveClass("opacity-100");
    expect(view.getByRole("alert")).toHaveTextContent("The renderer stoppedNo GPU adapter");
    expect(view.queryByRole("progressbar")).not.toBeInTheDocument();
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

    act(() => viewport.noteCamera(camera()));

    expect(await view.findByText("x -243.8 y 12.5 z 87.3")).toBeInTheDocument();
    expect(view.getByText("h 90.0° p 0.0°")).toBeInTheDocument();
  });

  // A clean view of the level is the reason to turn them off, and half a clean view is no use, so one switch takes
  // both away.
  it("takes both readouts off the viewport together", async () => {
    const { view, viewport } = renderReporting();

    act(() => viewport.noteCamera(camera()));

    expect(view.getByTestId("level-preview-metrics")).toBeInTheDocument();
    expect(view.getByTestId("level-preview-coordinates")).toBeInTheDocument();

    await userEvent.click(within(await openOverlays(view)).getByRole("checkbox", { name: "Readouts" }));

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
      act(() => viewport.noteCamera(camera({ x: tick })));
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

    expect(await view.findByText("B 260MB W 2GB")).toBeInTheDocument();
  });

  // With the settings timing the passes, the readout lists them as the renderer reports them.
  it("lists each pass's GPU time in the readout while the renderer reports them timed", () => {
    const { renderer, view, viewport } = renderReporting();

    act(() => {
      viewport.noteCamera(camera());
      runInAction(() => {
        renderer.frame = {
          ...EMPTY_RENDER_FRAME_REPORT,
          isGpuTimed: true,
          passes: [{ gpuTime: 0.5, name: "gbuffer" }],
        };
      });
    });

    expect(view.getByTestId("render-frame-passes")).toHaveTextContent("GPU0.50 msgbuffer0.50");

    act(() => {
      runInAction(() => {
        renderer.frame = EMPTY_RENDER_FRAME_REPORT;
      });
    });

    expect(view.queryByTestId("render-frame-passes")).not.toBeInTheDocument();
  });

  // One switch, offered where the readout is and where the settings are: either place sets it, and both show it.
  it("times the passes from the overlays' popover and from the settings alike", async () => {
    window.localStorage.clear();

    const container: Container = mockContainer([
      LevelLoadService,
      LevelRenderService,
      LevelViewService,
      LevelViewportService,
      LevelWeatherService,
      LevelLookService,
    ]);
    const settings: SettingsService = container.get(SettingsService);
    const view: RenderResult = renderWithProviders(
      <>
        <LevelPreviewLayout name={"levels\\zaton"} renderViewport={() => <div data-testid={"stub-viewport"} />} />
        <SettingsRendererDisplay />
      </>,
      { container, route: "/level-viewer" }
    );

    expect(settings.isGpuTimed).toBe(false);

    await userEvent.click(within(await openOverlays(view)).getByRole("checkbox", { name: "GPU time per pass" }));

    expect(settings.isGpuTimed).toBe(true);

    await closeOverlays(view);

    expect(view.getByRole("checkbox", { name: "GPU time per pass" })).toBeChecked();

    await userEvent.click(view.getByRole("checkbox", { name: "GPU time per pass" }));

    expect(settings.isGpuTimed).toBe(false);
    expect(within(await openOverlays(view)).getByRole("checkbox", { name: "GPU time per pass" })).not.toBeChecked();
  });

  // A sector lands many times a second while a level is read, and only the cover and the status bar say so.
  it("redraws nothing that draws the level as sectors arrive", async () => {
    let renders: number = 0;
    const { view, viewport } = renderReporting(() => {
      renders += 1;
    });

    await view.findByTestId("stub-viewport");

    const before: number = renders;

    for (let loaded = 0; loaded <= 20; loaded += 1) {
      act(() => viewport.noteLoad(toLoad(loaded)));
    }

    expect(await view.findByText("Reading sectors, 20 of 24")).toBeInTheDocument();
    expect(renders).toBe(before);
  });

  // The renderer's read comes first: it is what is drawn, and the objects are only listed beside it.
  it("says the spawned objects are being listed once the renderer has read the level", async () => {
    const { loader, view, viewport } = renderReporting(() => undefined);

    await view.findByTestId("stub-viewport");

    act(() => {
      runInAction(() => {
        loader.spawnReport = EMPTY_LEVEL_SPAWN_REPORT;
      });
      viewport.noteLoad(toLoad(3));
    });

    expect(await view.findByText("Reading sectors, 3 of 24")).toBeInTheDocument();

    act(() => viewport.noteLoad({ ...toLoad(24, 40), isReady: true }));

    expect(await view.findByText("Listing spawned objects")).toBeInTheDocument();
  });
});
