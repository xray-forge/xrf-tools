import { expect } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { getMockChannels, listMockInvokeCalls, MockChannel, mockIpcCommands } from "@xrf/ipc/testing";
import { ViewportEvent } from "@xrf/ipc/types/xrf-app-render";

import { mockLevelPartResponses, mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { mockLevelWeatherDescription } from "@/core/level/fixtures/weather.mocks";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";

/** The viewport and the scene the mocked renderer answers with. */
export const LEVEL_VIEWPORT: number = 7;
export const LEVEL_SCENE: number = 3;

/** The kinds of the overlays last sent, in order. */
export function sentOverlayKinds(): Array<string> {
  return (listMockInvokeCalls(ERenderCommand.SET_OVERLAYS).at(-1)?.overlays ?? []).map(({ kind }) => kind);
}

/**
 * A level open and provisioned as the tool provisions it, its render service attached to `element`.
 *
 * @param element - Where the viewport is attached.
 * @returns The container and the attached service.
 */
export async function mockAttachedLevelRender(
  element: HTMLElement = document.createElement("div")
): Promise<{ container: Container; service: LevelRenderService }> {
  mockIpcCommands({
    [ELevelsCommand.RESTORE_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
    ...mockLevelPartResponses({ start: { direction: { x: 0, y: 0, z: -1 }, position: { x: 10, y: 2, z: -5 } } }),
    [ELevelsCommand.DESCRIBE_WEATHER]: mockSessionResponse(mockLevelWeatherDescription()),
    [ERenderCommand.ATTACH_VIEWPORT]: LEVEL_VIEWPORT,
    [ELevelsCommand.SHOW_LEVEL]: LEVEL_SCENE,
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
    LevelRenderService,
  ]);

  // Provisioned as the tool provisions it, so the level's look and weather open with the level.
  container.provision();
  await waitFor(() => expect(container.get(LevelLoadService).selection.isRestored).toBe(true));

  const service: LevelRenderService = container.get(LevelRenderService);

  service.attach(element);
  // Attaching ends with the level shown.
  await waitFor(() => expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toHaveLength(1));

  return { container, service };
}

/** Tells the attached viewport's channel an event. */
export function emitViewportEvent(event: ViewportEvent): void {
  (getMockChannels()[0] as MockChannel<ViewportEvent>).onmessage(event);
}
