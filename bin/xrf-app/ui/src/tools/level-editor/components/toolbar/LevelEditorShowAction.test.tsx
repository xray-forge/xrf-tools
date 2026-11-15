import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ESceneToolKind, SCENE_HELPER_TOOLS } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { clearLocalStorage } from "@/core/storage";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockScenePartResponses, mockSelectedSceneDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import { isLevelEditorHelperOn } from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorShowAction } from "./LevelEditorShowAction";

interface IRenderedShowAction {
  container: Container;
  rendered: RenderResult;
}

/**
 * The fixture scene open, its three spawn points helped, and the Show action over it.
 *
 * @returns The rendered action and its container.
 */
async function renderShowAction(): Promise<IRenderedShowAction> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(),
  });

  const container: Container = mockContainer([LevelEditorSceneService, LevelEditorHelpersService]);

  await container.get(LevelEditorSceneService).open({
    engine: EXrayEngineChoice.AUTO,
    gameRoots: null,
    isDltx: false,
    objectsRoots: null,
    source: { path: "C:\\rawdata\\levels\\x8.level" },
  });
  await flush();

  return {
    container,
    rendered: renderWithProviders(<LevelEditorShowAction />, { container, route: "/level-editor" }),
  };
}

async function openShow(rendered: RenderResult): Promise<HTMLElement> {
  await userEvent.click(rendered.getByRole("button", { name: "Show" }));

  return rendered.findByRole("dialog", { name: "Show" });
}

describe("LevelEditorShowAction", () => {
  beforeEach(() => {
    resetMockInvoke();
    clearLocalStorage();
  });

  it("says how many tools' helpers it draws without being opened", async () => {
    const { rendered } = await renderShowAction();

    expect(rendered.getByRole("button", { name: "Show" })).toHaveAccessibleDescription(
      `Helpers of ${SCENE_HELPER_TOOLS.length} of ${SCENE_HELPER_TOOLS.length} tools`
    );
  });

  it("lists every helped tool with how many of the scene's objects it helps", async () => {
    const { rendered } = await renderShowAction();
    const dialog: HTMLElement = await openShow(rendered);
    const rows: Array<HTMLElement> = within(dialog).getAllByTestId("level-editor-helper-tool-row");

    expect(rows).toHaveLength(SCENE_HELPER_TOOLS.length);
    expect(within(dialog).getByRole("checkbox", { name: "Spawn points" })).toBeChecked();
    expect(rows.find((row: HTMLElement) => row.textContent?.includes("Spawn points"))).toHaveTextContent(/3$/);
    expect(rows.find((row: HTMLElement) => row.textContent?.includes("Ways"))).toHaveTextContent(/0$/);
  });

  it("switches one tool's helpers, and every tool's with the master switch", async () => {
    const { container, rendered } = await renderShowAction();
    const helpersService: LevelEditorHelpersService = container.get(LevelEditorHelpersService);
    const dialog: HTMLElement = await openShow(rendered);

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Ways" }));

    expect(isLevelEditorHelperOn(helpersService.helpers, ESceneToolKind.WAY)).toBe(false);
    expect(isLevelEditorHelperOn(helpersService.helpers, ESceneToolKind.SPAWN)).toBe(true);

    await userEvent.click(within(dialog).getByRole("checkbox", { name: "Helpers" }));

    expect(helpersService.helpers.isShown).toBe(false);
    expect(within(dialog).getByRole("checkbox", { name: "Spawn points" })).toBeDisabled();
    expect(rendered.getByRole("button", { name: "Show", hidden: true })).toHaveAccessibleDescription("No helpers");
  });
});
