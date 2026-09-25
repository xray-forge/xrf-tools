import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { DEFAULT_RENDERER_SHADOW_SETTINGS } from "@xrf/renderer";

import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { DEFAULT_LEVEL_FEATURE_OPTIONS, ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelShadowAction", () => {
  it("sets the view's own cascades over the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelShadowAction
        isOn
        shadows={DEFAULT_RENDERER_SHADOW_SETTINGS}
        features={DEFAULT_LEVEL_FEATURE_OPTIONS}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    expect(getByRole("button", { name: "Shadows" })).toHaveAccessibleDescription(
      "Shadows in 3 cascades, the widest 160 m across, at 2048. Right-click for its settings"
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Shadows" }) });
    await findByRole("dialog", { name: "Shadows" });
    await userEvent.click(getByRole("button", { name: "1" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...DEFAULT_LEVEL_FEATURE_OPTIONS, shadows: { cascades: [20] } });
  });

  it("is disabled, with the reason, while the settings draw no shadows", () => {
    const { getByRole } = renderWithProviders(
      <LevelShadowAction
        isOn
        isAvailable={false}
        shadows={DEFAULT_RENDERER_SHADOW_SETTINGS}
        features={DEFAULT_LEVEL_FEATURE_OPTIONS}
        onToggle={() => {}}
        onChange={() => {}}
      />
    );

    expect(getByRole("button", { name: "Shadows" })).toBeDisabled();
  });
});
