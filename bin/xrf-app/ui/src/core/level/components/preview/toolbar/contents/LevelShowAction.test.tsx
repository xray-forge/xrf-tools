import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ELevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";

import { LevelShowAction } from "@/core/level/components/preview/toolbar/contents/LevelShowAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { clearLocalStorage } from "@/core/storage";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelShowAction", () => {
  it("names the spawned kinds shown, and turns each over by its own checkbox", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setOption("isSpawnedItems", false);
    descriptor.view.setOption("isSpawnedWeapons", false);

    const { getByRole, findByRole, view } = renderLevelView(<LevelShowAction />, descriptor);

    expect(getByRole("button", { name: "Show" })).toHaveAccessibleDescription("Spawned props, lamps");

    await userEvent.click(getByRole("button", { name: "Show" }));
    await findByRole("dialog", { name: "Show" });

    expect(getByRole("checkbox", { name: "Items" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Items" }));
    await userEvent.click(getByRole("checkbox", { name: "Lamps" }));

    expect(view.options.isSpawnedItems).toBe(true);
    expect(view.options.isSpawnedLamps).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isSpawnedLamps);
  });

  // A kind drawn as nothing has only its helper, which waits on helpers being drawn at all.
  it("switches a volume's helper once helpers are on", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole, view } = renderLevelView(<LevelShowAction />, descriptor);

    await userEvent.click(getByRole("button", { name: "Show" }));
    await findByRole("dialog", { name: "Show" });

    expect(getByRole("checkbox", { name: "Restrictors helpers" })).toBeDisabled();

    act(() => view.toggleHelpers());

    const helper: HTMLElement = await findByRole("checkbox", { name: "Restrictors helpers" });

    expect(helper).toBeChecked();
    await userEvent.click(helper);
    expect(view.helpers.kinds).toEqual({ [ELevelObjectKind.RESTRICTORS]: false });
  });
});
