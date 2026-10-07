import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { ERenderGlassMode } from "@/core/ipc/types/xrf-renderer";
import { LevelGlassSettings } from "@/core/level/components/preview/LevelGlassSettings";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_RENDER_GLASS_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderGlassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelGlassSettings", () => {
  it("turns the view's glass to enhanced, offering its refraction only then, and back to the settings", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();

    function render(value: TRenderGlassSettings): ReactElement {
      return <LevelGlassSettings value={value} features={mockLevelFeatureOptions()} onChange={onChange} />;
    }

    const { getByRole, queryByRole, rerender } = renderWithProviders(render(DEFAULT_RENDER_GLASS_SETTINGS));

    expect(queryByRole("slider", { name: "Refraction" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      glass: { mode: ERenderGlassMode.ENHANCED },
    });

    rerender(render({ ...DEFAULT_RENDER_GLASS_SETTINGS, mode: ERenderGlassMode.ENHANCED }));

    expect(getByRole("slider", { name: "Refraction" })).toHaveAttribute("aria-valuetext", "1.00");

    await userEvent.click(getByRole("button", { name: "Back to the settings for the glass" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), glass: {} });
  });
});
