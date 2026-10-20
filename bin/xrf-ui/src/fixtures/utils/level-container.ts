import { expect } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { Binding, Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import { ReactNode } from "react";

import { LevelWeatherDescription, SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { RenderAppliedReport, RenderFrameReport } from "@/core/ipc/types/xrf-renderer";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { EMPTY_RENDER_FRAME_REPORT } from "@/core/render/lib/native/native-frame-report";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockRenderAppliedReport } from "@/fixtures/mocks/render.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

export interface ILevelViewContainerDescriptor {
  container: Container;
  load: LevelLoadService;
  look: LevelLookService;
  view: LevelViewService;
  viewport: LevelViewportService;
  weather: LevelWeatherService;
  render: LevelRenderService;
  renderSettings: RenderSettingsService;
}

/**
 * Builds the services a level view's toolbar and panels read, as the level viewer binds them.
 *
 * @param bindings - Services of the application over the view's, such as the level viewer's own.
 * @returns The container, and the services a test sets state on and reads back.
 */
export function mockLevelViewContainer(bindings: Array<Binding> = []): ILevelViewContainerDescriptor {
  const container: Container = mockContainer([
    LevelLoadService,
    LevelLookService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelRenderService,
    ...bindings,
  ]);

  return {
    container,
    load: container.get(LevelLoadService),
    look: container.get(LevelLookService),
    render: container.get(LevelRenderService),
    renderSettings: container.get(RenderSettingsService),
    view: container.get(LevelViewService),
    viewport: container.get(LevelViewportService),
    weather: container.get(LevelWeatherService),
  };
}

/** What a level view rendered beside the services behind it, `container` naming the services' container. */
export type TLevelViewRenderResult = ILevelViewContainerDescriptor & Omit<RenderResult, "container">;

/**
 * Renders a part of the level view with the services given.
 *
 * @param ui - What to render.
 * @param descriptor - The services it reads, set up as the case needs.
 * @returns The render result and the services, to query the one and read the other back.
 */
export function renderLevelView(ui: ReactNode, descriptor: ILevelViewContainerDescriptor): TLevelViewRenderResult {
  return { ...renderWithProviders(ui, { container: descriptor.container }), ...descriptor };
}

/** The level a view opens, and the weather its game names. */
export interface ILevelViewOpening {
  level?: Partial<SelectedLevelDescription>;
  weather?: Partial<LevelWeatherDescription>;
}

/**
 * Renders a part of the level view over an open level, once its weather is read.
 *
 * @param ui - What to render.
 * @param descriptor - The services it reads, set up as the case needs.
 * @param opening - The level and the weather the backend answers with.
 * @returns The render result and the services.
 */
export async function renderOpenLevelView(
  ui: ReactNode,
  descriptor: ILevelViewContainerDescriptor,
  opening: ILevelViewOpening = {}
): Promise<TLevelViewRenderResult> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription(opening.level)),
    ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription(opening.weather)),
  });

  const rendered: TLevelViewRenderResult = renderLevelView(ui, descriptor);

  await waitFor(() => expect(descriptor.weather.description.isReady).toBe(true));
  await waitFor(() => expect(descriptor.weather.cycle.isLoading).toBe(false));

  return rendered;
}

/**
 * Stands in for a frame the viewport reported, the rest of it empty.
 *
 * @param render - The service hearing the viewport.
 * @param report - What the frame says, such as the GPU it was drawn on and what that GPU can do.
 */
export function noteLevelFrame(render: LevelRenderService, report: Partial<RenderFrameReport>): void {
  act(() => {
    runInAction(() => {
      render.frame = { ...EMPTY_RENDER_FRAME_REPORT, ...report };
    });
  });
}

/**
 * Stands in for what the viewport last applied, the rest of it as nothing drawn.
 *
 * @param render - The service hearing the viewport.
 * @param report - What it applied, such as the sun or moon the sky draws.
 */
export function noteLevelApplied(render: LevelRenderService, report: Partial<RenderAppliedReport>): void {
  act(() => {
    runInAction(() => {
      render.applied = mockRenderAppliedReport(report);
    });
  });
}
