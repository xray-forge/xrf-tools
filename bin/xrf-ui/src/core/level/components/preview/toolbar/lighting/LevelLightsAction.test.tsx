import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

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

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Lights" }) });
    await findByRole("dialog", { name: "Lights" });
    await userEvent.click(getByRole("button", { name: "Back to the settings" }));

    expect(view.features).toEqual({ ...before, lights: {} });
  });

  it("opens on a click, saying why, while the settings keep the lights off", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ lights: { isEnabled: false } });

    const { getByRole, findByRole } = renderLevelView(<LevelLightsAction />, descriptor);

    expect(getByRole("button", { name: "Lights" })).toHaveAccessibleDescription(
      "Lights are off in Settings, under Rendering"
    );

    await userEvent.click(getByRole("button", { name: "Lights" }));

    expect(await findByRole("dialog", { name: "Lights" })).toHaveTextContent(
      "Lights are off in Settings, under Rendering"
    );
  });
});
