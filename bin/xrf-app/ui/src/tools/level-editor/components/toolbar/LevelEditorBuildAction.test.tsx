import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { EAssetsCommand } from "@xrf/ipc/commands/assets";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import { EXrayRootKind, XrayRootKind } from "@xrf/ipc/types/xrf-vfs";

import { createRoots } from "@/core/assets/lib/roots";
import { ELevelCompilerField } from "@/core/level/lib/compiler/level-compiler-field";
import { EToolId } from "@/core/routing/tool";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue } from "@/core/storage";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockScenePartResponses, mockSelectedSceneDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorBuildAction } from "./LevelEditorBuildAction";

/** The game root the scene was opened over besides the gamedata beside its rawdata. */
const GAME_ROOT: string = "C:\\Games\\CoP";

function handed(field: ELevelCompilerField): unknown {
  return getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, field));
}

/**
 * Opens the scene over {@link GAME_ROOT}, its probe answering `kind` or failing where null, and builds it.
 */
async function build(kind: XrayRootKind | null): Promise<void> {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(
      mockSelectedSceneDescription({
        game: { engine: EXrayEngineChoice.AUTO, isDltx: false, roots: createRoots([GAME_ROOT]) },
      })
    ),
    [EAssetsCommand.PROBE_ROOT]: () => {
      if (kind === null) {
        throw new Error("The root cannot be read");
      }

      return { kind, evidence: [], mounts: 1 };
    },
    ...mockScenePartResponses(),
  });

  const container: Container = mockContainer([LevelEditorSceneService]);

  await container.get(LevelEditorSceneService).open({
    engine: EXrayEngineChoice.AUTO,
    gameRoots: null,
    isDltx: false,
    objectsRoots: null,
    source: { path: "C:\\rawdata\\levels\\labx8.level" },
  });
  await flush();

  const { getByRole } = renderWithProviders(<LevelEditorBuildAction />, { container, route: "/level-editor" });

  await userEvent.click(getByRole("button", { name: "Build" }));
  await waitFor(() => expect(handed(ELevelCompilerField.SOURCE)).toBe("C:\\rawdata\\levels\\labx8.level"));
}

describe("LevelEditorBuildAction", () => {
  beforeEach(() => {
    resetMockInvoke();
    clearLocalStorage();
  });

  it("hands an installation root's gamedata and its game materials, as the probe names it", async () => {
    await build(EXrayRootKind.INSTALLATION);

    expect(handed(ELevelCompilerField.GAMEDATA)).toBe("C:\\Games\\CoP\\gamedata");
    expect(handed(ELevelCompilerField.GAME_MATERIALS)).toBe("C:\\Games\\CoP\\gamedata\\gamemtl.xr");
  });

  it("takes the root as game data where the probe fails", async () => {
    await build(null);

    expect(handed(ELevelCompilerField.GAMEDATA)).toBe(GAME_ROOT);
    expect(handed(ELevelCompilerField.GAME_MATERIALS)).toBe("C:\\Games\\CoP\\gamemtl.xr");
  });
});
