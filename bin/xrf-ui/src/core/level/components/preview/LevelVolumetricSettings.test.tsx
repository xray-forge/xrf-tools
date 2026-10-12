import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { ERenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { LevelVolumetricSettings } from "@/core/level/components/preview/LevelVolumetricSettings";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_RENDER_VOLUMETRIC_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderVolumetricSettings } from "@/core/render/lib/settings/render-feature-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelVolumetricSettings", () => {
  it("turns the view's volumetric lighting to enhanced, offering its strengths only then, and back to the settings", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();

    function render(value: TRenderVolumetricSettings): ReactElement {
      return <LevelVolumetricSettings value={value} features={mockLevelFeatureOptions()} onChange={onChange} />;
    }

    const { getByRole, queryByRole, rerender } = renderWithProviders(render(DEFAULT_RENDER_VOLUMETRIC_SETTINGS));

    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      volumetric: { mode: ERenderVolumetricMode.ENHANCED },
    });

    rerender(render({ ...DEFAULT_RENDER_VOLUMETRIC_SETTINGS, mode: ERenderVolumetricMode.ENHANCED }));

    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "0.50");
    expect(getByRole("slider", { name: "Anisotropy" })).toHaveAttribute("aria-valuetext", "0.15");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "120 m");

    await userEvent.click(getByRole("button", { name: "Back to the settings for the volumetric lighting" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), volumetric: {} });
  });
});
