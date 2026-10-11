import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { LevelVolumetricSettings } from "@/core/level/components/preview/LevelVolumetricSettings";
import { mockLevelViewContainer } from "@/fixtures/utils/level-container";
import { renderWithProviders } from "@/fixtures/utils/render";

beforeEach(() => {
  window.localStorage.clear();
});

describe("LevelVolumetricSettings", () => {
  it("turns the view's volumetric lighting to enhanced, offering its strengths only then, and back to the settings", async () => {
    const { container, renderSettings, view } = mockLevelViewContainer();

    // The settings draw the engine's, so the view's own choice is what turns it enhanced.
    renderSettings.setRendererOverrides({ volumetric: { mode: ERenderVolumetricMode.ENGINE } });

    const { getByRole, queryByRole } = renderWithProviders(<LevelVolumetricSettings />, { container });

    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.volumetric).toEqual({ mode: ERenderVolumetricMode.ENHANCED });
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "0.20");
    expect(getByRole("slider", { name: "Anisotropy" })).toHaveAttribute("aria-valuetext", "0.25");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "120 m");

    await userEvent.click(getByRole("button", { name: "Back to the settings for the volumetric lighting" }));

    expect(view.features.volumetric).toEqual({});
    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();
  });
});
