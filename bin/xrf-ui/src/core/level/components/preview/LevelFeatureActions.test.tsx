import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { LevelGrassAction } from "@/core/level/components/preview/LevelGrassAction";
import { LevelLightsAction } from "@/core/level/components/preview/LevelLightsAction";
import { ILevelFeatureOptions, TLevelFeatureKey } from "@/core/level/lib/features";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import { mockLevelViewContainer } from "@/fixtures/utils/level-container";
import { renderWithProviders } from "@/fixtures/utils/render";

/** One feature popover, how it is named, what the view sets over it, and what its settings say while off. */
interface IFeatureActionCase {
  key: Exclude<TLevelFeatureKey, "shadows" | "ambientOcclusion">;
  label: string;
  unavailable: string;
  features: Partial<ILevelFeatureOptions>;
  off: IRenderFeatureOverrides;
  action: ReactElement;
}

const CASES: ReadonlyArray<IFeatureActionCase> = [
  {
    action: <LevelGrassAction />,
    features: { grass: { radius: 80 } },
    key: "grass",
    label: "Grass",
    off: { grass: { isEnabled: false } },
    unavailable: "Grass is off in Settings, under Rendering",
  },
  {
    action: <LevelLightsAction />,
    features: { lights: { isShadowed: false } },
    key: "lights",
    label: "Lights",
    off: { lights: { isEnabled: false } },
    unavailable: "Lights are off in Settings, under Rendering",
  },
];

beforeEach(() => {
  window.localStorage.clear();
});

describe("level feature actions", () => {
  it.each(CASES)(
    "goes back to the settings for $label, leaving the other groups alone",
    async ({ key, label, features, action }) => {
      const { container, view } = mockLevelViewContainer();
      const before: ILevelFeatureOptions = { ...view.features, ...features };

      view.setFeatures(before);

      const { getByRole, findByRole } = renderWithProviders(action, { container });

      await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: label }) });
      await findByRole("dialog", { name: label });
      await userEvent.click(getByRole("button", { name: "Back to the settings" }));

      expect(view.features).toEqual({ ...before, [key]: {} });
    }
  );

  it.each(CASES)(
    "is disabled, with the reason, while the settings keep $label off",
    ({ label, unavailable, off, action }) => {
      const { container, renderSettings } = mockLevelViewContainer();

      renderSettings.setRendererOverrides(off);

      const { getByRole } = renderWithProviders(action, { container });

      expect(getByRole("button", { name: label })).toBeDisabled();
      expect(getByRole("button", { name: label })).toHaveAccessibleDescription(unavailable);
    }
  );
});
