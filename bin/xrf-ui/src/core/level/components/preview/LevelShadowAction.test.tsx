import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderContactShadowMode } from "@/core/ipc/types/xrf-renderer";
import { LevelShadowAction } from "@/core/level/components/preview/LevelShadowAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_RENDER_SHADOW_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderShadowSettings } from "@/core/render/lib/settings/render-feature-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelShadowAction", () => {
  it("sets the view's own cascades over the settings'", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelShadowAction
        isOn
        state={{ isAvailable: true, value: DEFAULT_RENDER_SHADOW_SETTINGS }}
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

  it("turns the view's contact shadows on, offering their strengths only then", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const enhanced: TRenderShadowSettings = {
      ...DEFAULT_RENDER_SHADOW_SETTINGS,
      contact: { ...DEFAULT_RENDER_SHADOW_SETTINGS.contact, mode: ERenderContactShadowMode.ENHANCED },
    };
    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      <LevelShadowAction
        isOn
        state={{ isAvailable: true, value: DEFAULT_RENDER_SHADOW_SETTINGS }}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Shadows" }) });
    await findByRole("dialog", { name: "Shadows" });

    expect(queryByRole("slider", { name: "Contact length" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), shadows: { contact: enhanced.contact } });

    rerender(
      <LevelShadowAction
        isOn
        state={{ isAvailable: true, value: enhanced }}
        features={mockLevelFeatureOptions()}
        onToggle={() => {}}
        onChange={onChange}
      />
    );

    expect(getByRole("slider", { name: "Contact length" })).toBeInTheDocument();
    expect(getByRole("button", { hidden: true, name: "Shadows" })).toHaveAccessibleDescription(
      "Shadows in 4 cascades, the widest 480 m across, at 2048, with contact shadows. Right-click for its settings"
    );
  });

  it("is disabled, with the reason, while the settings draw no shadows", () => {
    const { getByRole } = renderWithProviders(
      <LevelShadowAction
        isOn
        state={{ isAvailable: false, value: DEFAULT_RENDER_SHADOW_SETTINGS }}
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
