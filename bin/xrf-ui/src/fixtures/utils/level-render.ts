import { expect } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { ViewportEvent } from "@/core/ipc/types/xrf-app";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { getMockChannels, MockChannel, mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";

/** The viewport and the scene the mocked renderer answers with. */
export const LEVEL_VIEWPORT: number = 7;
export const LEVEL_SCENE: number = 3;

/** The arguments each call of a render command was sent with, in order. */
export function sentRenderCommand(command: string): Array<Record<string, unknown>> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === `plugin:render|${command}`)
    .map(([, args]) => args as Record<string, unknown>);
}

/** The kinds of the overlays last sent, in order. */
export function sentOverlayKinds(): Array<string> {
  return ((sentRenderCommand("set_overlays").at(-1)?.overlays ?? []) as Array<{ kind: string }>).map(
    ({ kind }) => kind
  );
}

/**
 * A level open and provisioned as the application provisions it, its render service attached to `element`.
 *
 * @param element - Where the viewport is attached.
 * @returns The container and the attached service.
 */
export async function mockAttachedLevelRender(
  element: HTMLElement = document.createElement("div")
): Promise<{ container: Container; service: LevelRenderService }> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(
      mockSelectedLevelDescription({ start: { direction: { x: 0, y: 0, z: -1 }, position: { x: 10, y: 2, z: -5 } } })
    ),
    ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    ["plugin:render|attach_viewport"]: LEVEL_VIEWPORT,
    ["plugin:render|show_level"]: LEVEL_SCENE,
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
    LevelRenderService,
  ]);

  // Provisioned as the application provisions it, so the level's look and weather open with the level.
  container.provision();
  await waitFor(() => expect(container.get(LevelLoadService).isReady).toBe(true));

  const service: LevelRenderService = container.get(LevelRenderService);

  service.attach(element);
  // Attaching ends with the level shown.
  await waitFor(() => expect(sentRenderCommand("show_level")).toHaveLength(1));

  return { container, service };
}

/** Tells the attached viewport's channel an event. */
export function emitViewportEvent(event: ViewportEvent): void {
  (getMockChannels()[0] as MockChannel<ViewportEvent>).onmessage(event);
}
