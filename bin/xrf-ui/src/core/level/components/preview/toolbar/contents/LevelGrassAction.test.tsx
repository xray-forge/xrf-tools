import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelGrassAction } from "@/core/level/components/preview/toolbar/contents/LevelGrassAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/fixtures/utils/level-container";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelGrassAction", () => {
  it("goes back to the settings for the grass, leaving the other groups alone", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const before: ILevelFeatureOptions = { ...descriptor.view.features, grass: { radius: 80 } };

    descriptor.view.setFeatures(before);

    const { getByRole, findByRole, view } = renderLevelView(<LevelGrassAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Grass" }) });
    await findByRole("dialog", { name: "Grass" });
    await userEvent.click(getByRole("button", { name: "Reset to the preset" }));

    expect(view.features).toEqual({ ...before, grass: {} });
  });

  it("opens on a click, saying why, while the settings keep the grass off", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ grass: { isEnabled: false } });

    const { getByRole, findByRole } = renderLevelView(<LevelGrassAction />, descriptor);

    expect(getByRole("button", { name: "Grass" })).toHaveAccessibleDescription(
      "Grass is off in Settings, under Rendering"
    );

    await userEvent.click(getByRole("button", { name: "Grass" }));

    expect(await findByRole("dialog", { name: "Grass" })).toHaveTextContent(
      "Grass is off in Settings, under Rendering"
    );
  });
});
