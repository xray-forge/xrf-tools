import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ESceneToolKind } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LEVEL_EDITOR_HELPERS_STORAGE_KEY } from "@/core/storage";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import {
  IMockSceneParts,
  MOCK_NO_SCENE_HELPERS,
  mockSceneDrawDescription,
  mockSceneObjectTool,
  mockScenePartResponses,
  mockSceneSpawn,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

import { LevelEditorViewport } from "./LevelEditorViewport";

async function renderViewport(parts: IMockSceneParts = {}): Promise<RenderResult> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(parts),
  });

  const container: Container = mockContainer([
    LevelEditorSceneService,
    LevelEditorSelectionService,
    LevelEditorHelpersService,
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
    localStorage.removeItem(LEVEL_EDITOR_HELPERS_STORAGE_KEY);
    resetMockInvoke();
  });

  it("says nothing over a scene that draws its objects", async () => {
    const { queryByTestId } = await renderViewport();

    expect(queryByTestId("level-editor-viewport-state")).toBeNull();
  });

  // A scene decompiled before its geometry holds helpers alone.
  it("says nothing over a scene that draws helpers alone", async () => {
    const { queryByTestId } = await renderViewport({
      drawing: { ...mockSceneDrawDescription(), bounds: null },
      objects: {
        problems: [],
        tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "actor", "actor")])],
      },
    });

    expect(queryByTestId("level-editor-viewport-state")).toBeNull();
  });

  it("says the helpers are switched off over a scene that draws helpers alone with none shown", async () => {
    localStorage.setItem(LEVEL_EDITOR_HELPERS_STORAGE_KEY, JSON.stringify({ isShown: false }));

    const { getByTestId } = await renderViewport({
      drawing: { ...mockSceneDrawDescription(), bounds: null },
      objects: {
        problems: [],
        tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "actor", "actor")])],
      },
    });

    expect(getByTestId("level-editor-viewport-state")).toHaveTextContent("its helpers are switched off");
  });

  it("says a scene holding no objects has nothing to draw", async () => {
    const { getByTestId } = await renderViewport({
      drawing: { ...mockSceneDrawDescription(), bounds: null },
      helpers: MOCK_NO_SCENE_HELPERS,
      objects: { problems: [], tools: [] },
    });

    expect(getByTestId("level-editor-viewport-state")).toHaveTextContent("This scene holds no objects");
  });

  it("says none of the scene's library objects was found where it draws neither them nor a helper", async () => {
    const { getByTestId } = await renderViewport({
      drawing: { ...mockSceneDrawDescription(), bounds: null },
      helpers: MOCK_NO_SCENE_HELPERS,
    });

    expect(getByTestId("level-editor-viewport-state")).toHaveTextContent(
      "None of this scene's library objects was found. Check the objects roots"
    );
  });
});
