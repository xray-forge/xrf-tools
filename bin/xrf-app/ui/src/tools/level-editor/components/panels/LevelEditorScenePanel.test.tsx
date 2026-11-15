import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ESceneToolKind, SceneToolDescription, SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import { Maybe } from "@xrf/types";

import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import {
  MOCK_SCENE_GUID,
  mockScenePartResponses,
  mockSceneToolDescription,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorScenePanel } from "./LevelEditorScenePanel";

/**
 * @param panel - The rendered panel.
 * @param label - A property's label.
 * @returns The property's value as the panel writes it.
 */
function readProperty(panel: HTMLElement, label: string): string {
  const property: Maybe<Element> = Array.from(panel.querySelectorAll("dl")).find(
    (it: Element) => it.querySelector("dt")?.textContent === label
  );

  expect(property).toBeDefined();

  return property?.querySelector("dd")?.textContent ?? "";
}

/**
 * The scene panel over `scene` once it is open.
 *
 * @param scene - What the open answers.
 * @returns The rendered panel.
 */
async function renderScene(scene: SelectedSceneDescription): Promise<RenderResult> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(scene),
    ...mockScenePartResponses(),
  });

  const container: Container = mockContainer([LevelEditorSceneService]);

  await container.get(LevelEditorSceneService).open({
    engine: EXrayEngineChoice.AUTO,
    gameRoots: null,
    isDltx: false,
    objectsRoots: null,
    source: scene.source,
  });
  await flush();

  return renderWithProviders(<LevelEditorScenePanel />, { container, route: "/level-editor" });
}

describe("LevelEditorScenePanel", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("says no scene is open before one is", () => {
    const { getByTestId } = renderWithProviders(<LevelEditorScenePanel />, {
      container: mockContainer([LevelEditorSceneService]),
      route: "/level-editor",
    });

    expect(getByTestId("level-editor-scene-panel").textContent).toContain("No scene open.");
  });

  it("shows where the scene lies, what its .level holds and the roots it reads from", async () => {
    const { getByTestId } = await renderScene(mockSelectedSceneDescription());
    const panel: HTMLElement = getByTestId("level-editor-scene-panel");

    expect(readProperty(panel, "Path")).toBe("C:\\rawdata\\levels\\labx8.level");
    expect(readProperty(panel, "GUID")).toBe(MOCK_SCENE_GUID);
    expect(readProperty(panel, "Objects roots")).toBe("C:\\rawdata\\objects");
    expect(readProperty(panel, "Game roots")).toBe("None: textures are missing");
    expect(readProperty(panel, "Weld distance")).toBe("0.01 m");
    expect(panel.textContent).toContain("3 tools");
  });

  it("names the GUID a tool's part file carries where it is not the scene's, and a file that is no part", async () => {
    const glows: SceneToolDescription = mockSceneToolDescription(ESceneToolKind.GLOW);
    const { getByTestId } = await renderScene(
      mockSelectedSceneDescription({
        tools: [
          mockSceneToolDescription(ESceneToolKind.SCENE_OBJECT),
          {
            ...glows,
            files: [
              { guid: "1:2", name: "glow.part", size: 1024 },
              { guid: null, name: "glow.part1", size: 12 },
            ],
          },
        ],
      })
    );
    const panel: HTMLElement = getByTestId("level-editor-scene-panel");

    expect(readProperty(panel, "Scene objects")).not.toContain("GUID");
    expect(readProperty(panel, "Glows")).toContain("glow.part · 1 KB · GUID 1:2");
    expect(readProperty(panel, "Glows")).toContain("glow.part1 · 12 B · not a part");
  });

  it("says the .level's build parameters and camera do not read where they do not", async () => {
    const { getByTestId } = await renderScene(mockSelectedSceneDescription({ buildParams: null, camera: null }));
    const panel: HTMLElement = getByTestId("level-editor-scene-panel");

    expect(panel.textContent).toContain("The scene's [build_params] do not read.");
    expect(panel.textContent).toContain("The scene's [camera] does not read.");
  });
});
