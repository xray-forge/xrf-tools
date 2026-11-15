import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ESceneToolKind } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import {
  IMockSceneParts,
  mockSceneDrawDescription,
  mockSceneObjectTool,
  mockScenePartResponses,
  mockSceneSpawn,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorViewport } from "./LevelEditorViewport";

/**
 * The fixture scene open with the parts given, its viewport over a stub surface.
 *
 * @param parts - What the scene's parts answer.
 * @returns The rendered viewport.
 */
async function renderViewport(parts: IMockSceneParts = {}): Promise<RenderResult> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(parts),
  });

  const container: Container = mockContainer([
    LevelEditorSceneService,
    LevelViewService,
    LevelViewportService,
    LevelEditorRenderService,
  ]);

  await container.get(LevelEditorSceneService).open({
    engine: EXrayEngineChoice.AUTO,
    gameRoots: null,
    isDltx: false,
    objectsRoots: null,
    source: { path: "C:\\rawdata\\levels\\x8.level" },
  });
  await flush();

  return renderWithProviders(
    <LevelEditorViewport isOpen={true} renderViewport={() => <div data-testid={"stub-viewport"} />} />,
    { container, route: "/level-editor" }
  );
}

describe("LevelEditorViewport", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("says nothing over a scene that draws its objects", async () => {
    const { queryByTestId } = await renderViewport();

    expect(queryByTestId("level-editor-viewport-state")).toBeNull();
  });

  // A scene decompiled before its geometry holds helpers alone, which are not drawn yet.
  it("says a scene holding no scene object has nothing to draw", async () => {
    const { getByTestId } = await renderViewport({
      drawing: { ...mockSceneDrawDescription(), bounds: null },
      objects: {
        problems: [],
        tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "actor", "actor")])],
      },
    });

    expect(getByTestId("level-editor-viewport-state")).toHaveTextContent(
      "This scene has no scene objects to draw. Its spawn points, ways and other tools are listed in the Outliner"
    );
  });

  it("says none of the scene's library objects was found where none is drawn", async () => {
    const { getByTestId } = await renderViewport({ drawing: { ...mockSceneDrawDescription(), bounds: null } });

    expect(getByTestId("level-editor-viewport-state")).toHaveTextContent(
      "None of this scene's library objects was found. Check the objects roots"
    );
  });
});
