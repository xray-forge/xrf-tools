import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERendererAntialiasing } from "@xrf/renderer";

import { LevelAntialiasingAction } from "@/core/level/components/preview/LevelAntialiasingAction";
import { DEFAULT_LEVEL_FEATURE_OPTIONS, ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelAntialiasingAction", () => {
  it("picks the view's own mode, and goes back to the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelAntialiasingAction
        isOn
        settingsMode={ERendererAntialiasing.SMAA}
        features={DEFAULT_LEVEL_FEATURE_OPTIONS}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Antialiasing" }) });
    await findByRole("dialog", { name: "Antialiasing" });
    await userEvent.click(getByRole("button", { name: "FXAA" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...DEFAULT_LEVEL_FEATURE_OPTIONS,
      antialiasing: ERendererAntialiasing.FXAA,
    });

    await userEvent.click(getByRole("button", { name: "Back to the settings" }));

    expect(onChange).toHaveBeenLastCalledWith(DEFAULT_LEVEL_FEATURE_OPTIONS);
  });

  it("is disabled while the settings smooth nothing", () => {
    const { getByRole } = renderWithProviders(
      <LevelAntialiasingAction
        isOn
        settingsMode={ERendererAntialiasing.NONE}
        features={DEFAULT_LEVEL_FEATURE_OPTIONS}
        onToggle={() => {}}
        onChange={() => {}}
      />
    );

    expect(getByRole("button", { name: "Antialiasing" })).toBeDisabled();
  });
});
