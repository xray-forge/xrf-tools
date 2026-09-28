import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { DEFAULT_RENDERER_SHADOW_SETTINGS } from "@xrf/renderer";

import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelShadowAction", () => {
  it("sets the view's own cascades over the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelShadowAction
        isOn
        state={{ isAvailable: true, value: DEFAULT_RENDERER_SHADOW_SETTINGS }}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    expect(getByRole("button", { name: "Shadows" })).toHaveAccessibleDescription(
      "Shadows in 4 cascades, the widest 480 m across, at 2048. Right-click for its settings"
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Shadows" }) });
    await findByRole("dialog", { name: "Shadows" });
    await userEvent.click(getByRole("button", { name: "1" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), shadows: { cascades: [20] } });
  });

  it("is disabled, with the reason, while the settings draw no shadows", () => {
    const { getByRole } = renderWithProviders(
      <LevelShadowAction
        isOn
        state={{ isAvailable: false, value: DEFAULT_RENDERER_SHADOW_SETTINGS }}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={() => {}}
      />
    );

    expect(getByRole("button", { name: "Shadows" })).toBeDisabled();
    expect(getByRole("button", { name: "Shadows" })).toHaveAccessibleDescription(
      "Shadows are off in Settings, under Rendering"
    );
  });
});
