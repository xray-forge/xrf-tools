import { expect } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { getMockChannels, IpcCommandMocks, listMockInvokeCalls, MockChannel, mockIpcCommands } from "@xrf/ipc/testing";
import { ViewportEvent } from "@xrf/ipc/types/xrf-app-render";

import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import {
  IMockSceneParts,
  mockScenePartResponses,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorObjectsService } from "@/tools/level-editor/services/level-editor-objects";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** The viewport the mocked renderer attaches. */
export const MOCK_SCENE_VIEWPORT: number = 5;

/** The session the fixture scene is restored in. */
export const MOCK_SCENE_SESSION: string = "fixture-session";

/** What the renderer answers of a resident scene it drew whole. */
export const MOCK_CLEAN_SCENE_RESIDENT: IpcCommandMocks = {
  [ERenderCommand.DESCRIBE_PROBLEMS]: { models: [], sectors: [], skipped: [] },
  [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [],
};

/** The editor's services as a test drives them, the render service attached. */
export interface IAttachedSceneRender {
  container: Container;
  sceneService: LevelEditorSceneService;
  service: LevelEditorRenderService;
}

/**
 * The fixture scene restored and provisioned as the editor provisions it, its render service attached; each show
 * answers a scene of its own, as the renderer starts one for every show of a viewport it alone draws.
 *
 * @param parts - What the scene's parts answer, each defaulting to the fixture's own.
 * @returns The container and the services a test drives.
 */
export async function mockAttachedSceneRender(parts: IMockSceneParts = {}): Promise<IAttachedSceneRender> {
  let scenes: number = 0;

  mockIpcCommands({
    [ELevelsCommand.RESTORE_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(parts),
    [ERenderCommand.ATTACH_VIEWPORT]: MOCK_SCENE_VIEWPORT,
    [ELevelsCommand.SHOW_SCENE]: () => (scenes += 1),
    ...MOCK_CLEAN_SCENE_RESIDENT,
  });

  const container: Container = mockContainer([
    LevelEditorSceneService,
    LevelEditorSelectionService,
    LevelEditorHelpersService,
    LevelViewService,
    LevelViewportService,
    LevelEditorRenderService,
    LevelEditorObjectsService,
  ]);
  const sceneService: LevelEditorSceneService = container.get(LevelEditorSceneService);

  container.provision();
  await waitFor(() => expect(sceneService.drawing.value).not.toBeNull());
  await waitFor(() => expect(sceneService.helpers.value).not.toBeNull());
  await waitFor(() => expect(sceneService.objects.value).not.toBeNull());

  const service: LevelEditorRenderService = container.get(LevelEditorRenderService);

  service.attach(document.createElement("div"));
  await waitFor(() => expect(listMockInvokeCalls(ELevelsCommand.SHOW_SCENE)).toHaveLength(1));

  return { container, sceneService, service };
}

/**
 * Tells the attached viewport's service what the renderer reported.
 *
 * @param event - What the viewport reported.
 */
export function emitSceneViewportEvent(event: ViewportEvent): void {
  (getMockChannels()[0] as MockChannel<ViewportEvent>).onmessage(event);
}
