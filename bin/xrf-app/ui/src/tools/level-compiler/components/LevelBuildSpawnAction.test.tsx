import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { EToolId } from "@/core/routing/tool";
import { ESpawnCompilerField } from "@/core/spawn/lib/compiler/spawn-compiler-field";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

import { LevelBuildSpawnAction } from "./LevelBuildSpawnAction";

function remembered(field: ESpawnCompilerField): unknown {
  return getLocalStorageValue(getFieldValueStorageKey(EToolId.SPAWN_COMPILER, field));
}

describe("LevelBuildSpawnAction", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  it("opens the spawn compiler holding the levels the level was built among, and the game data it was compiled with", async () => {
    const { getByRole } = renderWithProviders(
      <LevelBuildSpawnAction level={"D:\\qa\\labx8\\gamedata\\levels\\labx8"} gamedata={"C:\\Games\\CoP\\gamedata"} />,
      { route: "/level-compiler" }
    );

    await userEvent.click(getByRole("button", { name: "Build spawn" }));

    expect(remembered(ESpawnCompilerField.LEVELS_ROOT)).toBe("D:\\qa\\labx8\\gamedata\\levels");
    expect(remembered(ESpawnCompilerField.DESTINATION)).toBe("D:\\qa\\labx8\\gamedata\\spawns");
    expect(remembered(ESpawnCompilerField.GAMEDATA)).toBe("C:\\Games\\CoP\\gamedata");
  });

  it("keeps the game data the spawn compiler remembered where the build named none", async () => {
    setLocalStorageValue(getFieldValueStorageKey(EToolId.SPAWN_COMPILER, ESpawnCompilerField.GAMEDATA), "D:\\game");

    const { getByRole } = renderWithProviders(
      <LevelBuildSpawnAction level={"D:\\qa\\gamedata\\levels\\zaton"} gamedata={null} />,
      { route: "/level-compiler" }
    );

    await userEvent.click(getByRole("button", { name: "Build spawn" }));

    expect(remembered(ESpawnCompilerField.LEVELS_ROOT)).toBe("D:\\qa\\gamedata\\levels");
    expect(remembered(ESpawnCompilerField.GAMEDATA)).toBe("D:\\game");
  });

  it("offers nothing before a level was written", () => {
    const { getByRole } = renderWithProviders(<LevelBuildSpawnAction level={null} gamedata={null} />, {
      route: "/level-compiler",
    });

    expect(getByRole("button", { name: "Build spawn" })).toBeDisabled();
  });
});
