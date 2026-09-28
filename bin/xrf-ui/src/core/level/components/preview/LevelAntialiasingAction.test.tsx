import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERendererAntialiasing } from "@xrf/renderer";

import { LevelAntialiasingAction } from "@/core/level/components/preview/LevelAntialiasingAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelAntialiasingAction", () => {
  it("picks the view's own mode, and goes back to the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelAntialiasingAction
        isOn
        settingsMode={ERendererAntialiasing.SMAA}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Antialiasing" }) });
    await findByRole("dialog", { name: "Antialiasing" });
    await userEvent.click(getByRole("button", { name: "FXAA" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      antialiasing: ERendererAntialiasing.FXAA,
    });

    await userEvent.click(getByRole("button", { name: "Back to the settings" }));

    expect(onChange).toHaveBeenLastCalledWith(mockLevelFeatureOptions());
  });

  it("is disabled while the settings smooth nothing", () => {
    const { getByRole } = renderWithProviders(
      <LevelAntialiasingAction
        isOn
        settingsMode={ERendererAntialiasing.NONE}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
      />
    );

    expect(getByRole("button", { name: "Antialiasing" })).toBeDisabled();
  });
});
