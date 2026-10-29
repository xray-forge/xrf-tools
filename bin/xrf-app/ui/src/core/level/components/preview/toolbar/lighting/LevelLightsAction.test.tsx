import { beforeEach, describe, expect, it } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ERenderLightFalloff, ERenderVolumetricMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelLightsAction } from "@/core/level/components/preview/toolbar/lighting/LevelLightsAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { clearLocalStorage } from "@/core/storage";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelLightsAction", () => {
  it("goes back to the settings for the lights, leaving the other groups alone", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const before: ILevelFeatureOptions = { ...descriptor.view.features, lights: { isShadowed: false } };

    descriptor.view.setFeatures(before);

    const { getByRole, findByRole, view } = renderLevelView(<LevelLightsAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Lamps" }) });
    await findByRole("dialog", { name: "Lamps" });
    await userEvent.click(getByRole("button", { name: "Reset to the preset" }));

    expect(view.features).toEqual({ ...before, lights: {} });
  });

  it("opens on a click, saying why, while the settings keep the lights off", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ lights: { isEnabled: false } });

    const { getByRole, findByRole } = renderLevelView(<LevelLightsAction />, descriptor);

    expect(getByRole("button", { name: "Lamps" })).toHaveAccessibleDescription(
      "Lamps are off in Settings, under Rendering"
    );

    await userEvent.click(getByRole("button", { name: "Lamps" }));

    expect(await findByRole("dialog", { name: "Lamps" })).toHaveTextContent(
      "Lamps are off in Settings, under Rendering"
    );
  });

  it("lets the lamps fall off by the game's curve on asking, physically by default", async () => {
    const { getByRole, findByRole, view } = renderLevelView(<LevelLightsAction />, mockLevelViewContainer());

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Lamps" }) });
    await findByRole("dialog", { name: "Lamps" });

    const falloff: HTMLElement = getByRole("group", { name: "Falloff" });

    expect(within(falloff).getByRole("button", { name: "Physical" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(within(falloff).getByRole("button", { name: "Engine" }));

    expect(view.features.lights).toEqual({ falloff: ERenderLightFalloff.ENGINE });
  });

  it("sets the view's volumetric light beside the lamps' shafts it replaces", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ volumetric: { mode: ERenderVolumetricMode.ENGINE } });

    const { getByRole, findByRole, view } = renderLevelView(<LevelLightsAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Lamps" }) });
    await findByRole("dialog", { name: "Lamps" });
    await userEvent.click(getByRole("checkbox", { name: "Enhanced volumetric light" }));

    expect(view.features.volumetric).toEqual({ mode: ERenderVolumetricMode.ENHANCED });
  });
});
