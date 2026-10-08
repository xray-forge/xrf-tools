import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { ERenderRainMode } from "@/core/ipc/types/xrf-renderer";
import { LevelWetSurfacesAction } from "@/core/level/components/preview/LevelWetSurfacesAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_RENDER_RAIN_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelWetSurfacesAction", () => {
  it("turns the view's rain to enhanced, offering its strengths only then, and back to the settings", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const enhanced: TRenderRainSettings = { ...DEFAULT_RENDER_RAIN_SETTINGS, mode: ERenderRainMode.ENHANCED };

    function render(value: TRenderRainSettings): ReactElement {
      return <LevelWetSurfacesAction value={value} features={mockLevelFeatureOptions()} onChange={onChange} />;
    }

    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(render(DEFAULT_RENDER_RAIN_SETTINGS));

    expect(getByRole("button", { name: "Wet surfaces" })).toHaveAccessibleDescription(
      "The engine's splashes. Right-click for its settings"
    );

    await userEvent.click(getByRole("button", { name: "Wet surfaces" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      rain: { mode: ERenderRainMode.ENHANCED },
    });

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Wet surfaces" }) });
    await findByRole("dialog", { name: "Wet surfaces" });

    expect(queryByRole("slider", { name: "Puddles" })).toBeNull();

    rerender(render(enhanced));

    expect(getByRole("checkbox", { name: "Wet surfaces and puddles" })).toBeChecked();
    expect(getByRole("slider", { name: "Puddles" })).toHaveAttribute("aria-valuetext", "80%");
    expect(getByRole("slider", { name: "Reflectivity" })).toHaveAttribute("aria-valuetext", "40%");
    expect(getByRole("button", { hidden: true, name: "Wet surfaces" })).toHaveAccessibleDescription(
      "Wet surfaces and puddles, 80% of flat ground, 40% reflective. Right-click for its settings"
    );

    await userEvent.click(getByRole("button", { name: "Back to the settings for the rain" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), rain: {} });
  });
});
