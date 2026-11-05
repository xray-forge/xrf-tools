import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelGrassAction } from "@/core/level/components/preview/toolbar/world/LevelGrassAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { TRenderFoliageSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";

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
    await userEvent.click(getByRole("button", { name: "Grass: Back to the settings" }));

    expect(view.features).toEqual({ ...before, grass: {} });
  });

  // The foliage's motion is the Wind popover's, so the reset here neither offers to set it back nor does.
  it("offers no reset for the foliage's motion alone, and keeps it on setting the grass back", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const FOLIAGE: TRenderFoliageSettings = descriptor.renderSettings.rendererFeatures.grass.foliage;
    const moved: TRenderFoliageSettings = { ...FOLIAGE, grassSpeed: FOLIAGE.grassSpeed + 0.5 };

    descriptor.view.setFeatures({ ...descriptor.view.features, grass: { foliage: moved } });

    const { getByRole, findByRole, queryByRole, view } = renderLevelView(<LevelGrassAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Grass" }) });
    await findByRole("dialog", { name: "Grass" });

    expect(queryByRole("button", { name: "Grass: Back to the settings" })).toBeNull();

    act(() => view.setFeatures({ ...view.features, grass: { foliage: moved, radius: 80 } }));
    await userEvent.click(await findByRole("button", { name: "Grass: Back to the settings" }));

    expect(view.features.grass).toEqual({ foliage: moved });
  });

  it("keeps the shadows' reach settable only while the grass is shadowed", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.view.setFeatures({ ...descriptor.view.features, grass: { isShadowed: false } });

    const { getByRole, findByRole, view } = renderLevelView(<LevelGrassAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Grass" }) });
    await findByRole("dialog", { name: "Grass" });

    const shadows: HTMLElement = getByRole("group", { name: "Shadows" });

    expect(within(shadows).getByRole("slider", { name: "Radius" })).toBeDisabled();

    await userEvent.click(within(shadows).getByRole("checkbox", { name: "Shadows" }));

    expect(view.features.grass).toEqual({ isShadowed: true });
    expect(within(shadows).getByRole("slider", { name: "Radius" })).toBeEnabled();
    expect(within(shadows).getByRole("slider", { name: "Cascades" })).toBeEnabled();
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
