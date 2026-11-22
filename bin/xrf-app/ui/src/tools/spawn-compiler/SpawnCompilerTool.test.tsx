import { beforeEach, describe, expect, it } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { SpawnRootRequest } from "@xrf/ipc/types/xrf-spawn-builder";
import { Nullable } from "@xrf/types";

import { EToolId } from "@/core/routing/tool";
import { ESpawnCompilerField } from "@/core/spawn/lib/compiler/spawn-compiler-field";
import { clearLocalStorage, getFieldValueStorageKey, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockSpawnBuildResult, mockSpawnListedLevels } from "@/tools/spawn-compiler/fixtures/spawn-build.mocks";
import { SpawnCompilerService } from "@/tools/spawn-compiler/services/spawn-compiler.service";

import { SpawnCompilerTool } from "./SpawnCompilerTool";

function remember(field: ESpawnCompilerField, path: string): void {
  setLocalStorageValue(getFieldValueStorageKey(EToolId.SPAWN_COMPILER, field), path);
}

function render() {
  return renderWithProviders(<SpawnCompilerTool />, {
    route: "/spawn-compiler",
    bindings: [SpawnCompilerService],
  });
}

describe("SpawnCompilerTool", () => {
  beforeEach(() => {
    clearLocalStorage();
    remember(ESpawnCompilerField.GAMEDATA, "D:\\qa\\gamedata");
    remember(ESpawnCompilerField.LEVELS_ROOT, "D:\\qa\\gamedata\\levels");
    remember(ESpawnCompilerField.DESTINATION, "D:\\qa\\spawn");
  });

  it("offers the levels built, then builds the ones ticked from the start level and in the dialect chosen, and shows what it wrote", async () => {
    let sent: Nullable<SpawnRootRequest> = null;
    let sentDltx: Nullable<boolean> = null;

    mockIpcCommands({
      [ESpawnCommand.LIST_LEVELS]: mockSpawnListedLevels(),
      [ESpawnCommand.BUILD_SPAWN]: ({ request, isDltx }) => {
        sent = request;
        sentDltx = isDltx;

        return mockSpawnBuildResult([
          {
            ruleId: "spawn.connection.skipped",
            severity: "warning",
            subject: "zaton",
            message: "Graph point zat_to_jup connects to level jupiter, which is not built; the connection is left out",
          },
        ]);
      },
    });

    const { findByRole, getByRole, queryByRole, findByText } = render();

    fireEvent.click(await findByRole("checkbox", { name: "zaton" }));
    fireEvent.click(getByRole("checkbox", { name: "labx8" }));

    // Listed by the game but not built in the levels directory: not offered.
    expect(queryByRole("checkbox", { name: "jupiter" })).not.toBeInTheDocument();

    fireEvent.click(getByRole("option", { name: "labx8" }));
    fireEvent.change(getByRole("textbox", { name: "Spawn name" }), { target: { value: "qa_levels" } });
    fireEvent.click(getByRole("button", { name: "DLTX" }));

    const submit = getByRole("button", { name: "Build" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText("Built D:\\qa\\spawn\\qa_levels.spawn from zaton, labx8")).toBeInTheDocument();
    expect(await findByText(/connects to level jupiter, which is not built/)).toBeInTheDocument();
    expect(sent).toEqual({
      gamedata: "D:\\qa\\gamedata",
      levelsRoot: "D:\\qa\\gamedata\\levels",
      levels: ["zaton", "labx8"],
      destination: "D:\\qa\\spawn",
      name: "qa_levels",
      options: { start: "labx8", isSeparatorStrict: false },
      isForced: false,
    });
    expect(sentDltx).toBe(true);
  });

  it("shows why the build was refused", async () => {
    mockIpcCommands({
      [ESpawnCommand.LIST_LEVELS]: mockSpawnListedLevels(),
      [ESpawnCommand.BUILD_SPAWN]: () => {
        throw new Error("Level changer zat_to_jup on level zaton leads to level jupiter, which is not built");
      },
    });

    const { findByRole, getByRole, findByText } = render();

    fireEvent.click(await findByRole("checkbox", { name: "zaton" }));

    const submit = getByRole("button", { name: "Build" });

    await waitFor(() => expect(submit).toBeEnabled());
    fireEvent.click(submit);

    expect(await findByText(/leads to level jupiter, which is not built/)).toBeInTheDocument();
  });

  it("holds the build back until a level is ticked, and names why", async () => {
    mockIpcCommands({ [ESpawnCommand.LIST_LEVELS]: mockSpawnListedLevels() });

    const { findByRole, getByRole, getByText } = render();

    await findByRole("checkbox", { name: "zaton" });

    expect(getByRole("button", { name: "Build" })).toBeDisabled();
    expect(getByText("Tick the levels the spawn is built from")).toBeInTheDocument();
  });
});
