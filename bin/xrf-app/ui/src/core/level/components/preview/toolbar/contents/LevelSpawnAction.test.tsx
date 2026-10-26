import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelSpawnAction } from "@/core/level/components/preview/toolbar/contents/LevelSpawnAction";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/fixtures/utils/level-container";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelSpawnAction", () => {
  it("names the spawned categories shown, and turns each over by its own checkbox", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setOption("isSpawnedItems", false);
    descriptor.view.setOption("isSpawnedWeapons", false);

    const { getByRole, findByRole, view } = renderLevelView(<LevelSpawnAction />, descriptor);

    expect(getByRole("button", { name: "Spawn" })).toHaveAccessibleDescription("Spawned props, lamps");

    await userEvent.click(getByRole("button", { name: "Spawn" }));
    await findByRole("dialog", { name: "Spawn" });

    expect(getByRole("checkbox", { name: "Items" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Items" }));
    await userEvent.click(getByRole("checkbox", { name: "Lamps" }));

    expect(view.options.isSpawnedItems).toBe(true);
    expect(view.options.isSpawnedLamps).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isSpawnedLamps);
  });
});
