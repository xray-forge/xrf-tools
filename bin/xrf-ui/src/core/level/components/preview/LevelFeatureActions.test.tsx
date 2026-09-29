import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { DEFAULT_RENDERER_GRASS_SETTINGS, DEFAULT_RENDERER_LIGHTS_SETTINGS } from "@xrf/renderer";
import { ReactElement } from "react";

import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelLightsAction } from "@/core/level/components/preview/LevelLightsAction";
import { ILevelFeatureOptions, TLevelFeatureKey } from "@/core/level/lib/features";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

/** One feature popover, how it is named, and what its settings say while they keep it off. */
interface IFeatureActionCase {
  key: Exclude<TLevelFeatureKey, "shadows" | "ambientOcclusion">;
  label: string;
  unavailable: string;
  render: (isAvailable: boolean, onChange: (features: ILevelFeatureOptions) => void) => ReactElement;
}

const CASES: ReadonlyArray<IFeatureActionCase> = [
  {
    key: "grass",
    label: "Grass",
    render: (isAvailable, onChange) => (
      <LevelGrassAction
        isOn
        state={{ isAvailable, value: DEFAULT_RENDERER_GRASS_SETTINGS }}
        features={{ ...mockLevelFeatureOptions(), grass: { radius: 80 } }}
        onToggle={() => {}}
        onChange={onChange}
      />
    ),
    unavailable: "Grass is off in Settings, under Rendering",
  },
  {
    key: "lights",
    label: "Lights",
    render: (isAvailable, onChange) => (
      <LevelLightsAction
        isOn
        state={{ isAvailable, value: DEFAULT_RENDERER_LIGHTS_SETTINGS }}
        features={{ ...mockLevelFeatureOptions(), lights: { isShadowed: false } }}
        onToggle={() => {}}
        onChange={onChange}
      />
    ),
    unavailable: "Lights are off in Settings, under Rendering",
  },
];

describe("level feature actions", () => {
  it.each(CASES)(
    "goes back to the settings for $label, leaving the other groups alone",
    async ({ key, label, render }) => {
      const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
      const { getByRole, findByRole } = renderWithProviders(render(true, onChange));

      await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: label }) });
      await findByRole("dialog", { name: label });
      await userEvent.click(getByRole("button", { name: "Back to the settings" }));

      expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), [key]: {} });
    }
  );

  it.each(CASES)(
    "is disabled, with the reason, while the settings keep $label off",
    ({ label, unavailable, render }) => {
      const { getByRole } = renderWithProviders(render(false, () => {}));

      expect(getByRole("button", { name: label })).toBeDisabled();
      expect(getByRole("button", { name: label })).toHaveAccessibleDescription(unavailable);
    }
  );
});
