import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { AsyncState } from "@xrf/async";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import { ERenderTextureGroup } from "@xrf/ipc/types/xrf-renderer";

import { EMPTY_LEVEL_RESIDENT, ILevelResident } from "@/core/level/lib/render/level-resident";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { mockCommandAnswer } from "@/core/operations/fixtures/command-operation.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockScenePartResponses, mockSelectedSceneDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorRenderService } from "@/tools/level-editor/services/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorProblemsPanel } from "./LevelEditorProblemsPanel";

/** What the renderer says of a resident scene whose terrain texture the game data does not hold. */
const UNTEXTURED: ILevelResident = {
  ...EMPTY_LEVEL_RESIDENT,
  problems: {
    models: [{ name: "props\\box", reason: "Library object 'props\\box' does not compile: no surfaces" }],
    sectors: [],
    skipped: [],
  },
  textures: {
    ...EMPTY_LEVEL_RESIDENT.textures,
    problems: [
      {
        group: ERenderTextureGroup.SURFACES,
        reason: "Nothing in the mounted roots answers to it",
        reference: "terrain\\terrain_zaton",
      },
    ],
  },
};

/**
 * The fixture scene open, its parts read, and the problems panel over it.
 *
 * @param resident - How asking the renderer about the resident scene went.
 * @returns The rendered panel.
 */
async function renderProblems(resident: AsyncState<ILevelResident>): Promise<RenderResult> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(),
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
    source: { path: "C:\\rawdata\\levels\\zaton.level" },
  });
  await flush();

  mockCommandAnswer(container.get(LevelEditorRenderService).resident, resident);

  return renderWithProviders(<LevelEditorProblemsPanel />, { container, route: "/level-editor" });
}

describe("LevelEditorProblemsPanel", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("names a texture the game data answers nothing for, and a library object the renderer could not model", async () => {
    const { getByTestId } = await renderProblems(AsyncState.ready(UNTEXTURED));
    const panel: HTMLElement = getByTestId("level-editor-problems-panel");

    expect(panel.textContent).toContain("terrain_zaton");
    expect(panel.textContent).toContain("Nothing in the mounted roots answers to it");
    expect(panel.textContent).toContain("box_10 is not drawn: Library object 'props\\box' does not compile");
    expect(panel.textContent).not.toContain("No problems found");
  });

  // Before the scene is resident the renderer has said nothing, which is not the same as having found nothing.
  it("says the renderer's part is not read yet rather than calling the scene clean", async () => {
    const { getByTestId } = await renderProblems(AsyncState.idle());

    expect(getByTestId("editor-problems-status")).toHaveTextContent(
      "Not read yet: the renderer says what it could not draw once the scene is resident."
    );
    expect(getByTestId("level-editor-problems-panel").textContent).not.toContain("No problems found");
  });

  it("says what was checked once the renderer found nothing either", async () => {
    const { getByTestId } = await renderProblems(AsyncState.ready(EMPTY_LEVEL_RESIDENT));

    expect(getByTestId("level-editor-problems-panel").textContent).toContain(
      "No problems found. Every part file of this scene reads as its tool's"
    );
  });
});
