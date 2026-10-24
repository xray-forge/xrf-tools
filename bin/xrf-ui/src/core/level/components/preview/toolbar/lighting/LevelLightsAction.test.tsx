import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderVolumetricMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelLightsAction } from "@/core/level/components/preview/toolbar/lighting/LevelLightsAction";
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

  it("sets the view's volumetric light beside the lamps' shafts it replaces", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ volumetric: { mode: ERenderVolumetricMode.ENGINE } });

    const { getByRole, findByRole, view } = renderLevelView(<LevelLightsAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Lamps" }) });
    await findByRole("dialog", { name: "Lamps" });
    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.volumetric).toEqual({ mode: ERenderVolumetricMode.ENHANCED });
  });
});
