import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { LevelBuildResult } from "@xrf/ipc/types/xrf-app-levels";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { LevelCompileReport } from "@xrf/ipc/types/xrf-level-compiler";
import { Nullable } from "@xrf/types";

import { EToolId } from "@/core/routing/tool";
import { ESpawnCompilerField } from "@/core/spawn/lib/compiler/spawn-compiler-field";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelBuildResultView } from "./LevelBuildResultView";

const LEVEL: string = "D:\\qa\\gamedata\\levels\\zaton";
const GAMEDATA_KEY: string = getFieldValueStorageKey(EToolId.SPAWN_COMPILER, ESpawnCompilerField.GAMEDATA);

function result(compile: Nullable<LevelCompileReport>): LevelBuildResult {
  return {
    outcome: EJobOutcome.COMPLETED,
    report: { source: "D:\\qa\\zaton.level", destination: LEVEL, files: [], removed: [], geometry: null, refused: [] },
    compile,
  };
}

describe("LevelBuildResultView", () => {
  beforeEach(() => {
    clearLocalStorage();
    setLocalStorageValue(GAMEDATA_KEY, "D:\\remembered");
  });

  it("hands the spawn compiler the game data a compile of this run read", async () => {
    const { getByRole } = renderWithProviders(
      <LevelBuildResultView
        result={result({ level: LEVEL, steps: [] })}
        outputPath={LEVEL}
        gamedata={"C:\\Games\\CoP\\gamedata"}
      />,
      { route: "/level-compiler" }
    );

    await userEvent.click(getByRole("button", { name: "Build spawn" }));

    expect(getLocalStorageValue(GAMEDATA_KEY)).toBe("C:\\Games\\CoP\\gamedata");
  });

  // The form's game data is remembered from earlier runs; a build alone read none of it.
  it("hands no game data for a build that ran no compile", async () => {
    const { getByRole } = renderWithProviders(
      <LevelBuildResultView result={result(null)} outputPath={LEVEL} gamedata={"C:\\Games\\Other\\gamedata"} />,
      { route: "/level-compiler" }
    );

    await userEvent.click(getByRole("button", { name: "Build spawn" }));

    expect(getLocalStorageValue(GAMEDATA_KEY)).toBe("D:\\remembered");
  });
});
