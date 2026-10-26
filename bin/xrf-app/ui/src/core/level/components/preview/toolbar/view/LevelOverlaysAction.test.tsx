import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelOverlaysAction } from "@/core/level/components/preview/toolbar/view/LevelOverlaysAction";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/fixtures/utils/level-container";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelOverlaysAction", () => {
  it("names the overlays shown, and times the passes from the readouts", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setOption("isAxesVisible", true);

    const { getByRole, findByRole } = renderLevelView(<LevelOverlaysAction />, descriptor);

    expect(getByRole("button", { name: "Overlays" })).toHaveAccessibleDescription("Showing axes, readouts");

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Grid" })).not.toBeChecked();
  });

  // The marker says where the level's sunlight comes from, which the weather's own sun in the sky already shows by day.
  it("marks the sun among the overlays, off unless asked for", async () => {
    const { getByRole, findByRole, view } = renderLevelView(<LevelOverlaysAction />, mockLevelViewContainer());

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Sun" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Sun" }));

    expect(view.options.isSunMarked).toBe(true);
  });
});
