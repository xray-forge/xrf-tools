import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { ERenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";

import { LevelTerrainSettings } from "@/core/level/components/preview/toolbar/view/LevelTerrainSettings";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { DEFAULT_RENDER_TERRAIN_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelTerrainSettings", () => {
  it("shows the varied tiling's hexagons as the settings lay them, settable only while the tiling is varied", () => {
    const { container, renderSettings } = mockLevelViewContainer();

    renderSettings.setRendererOverrides({ terrain: DEFAULT_RENDER_TERRAIN_SETTINGS });

    const { getByRole } = renderWithProviders(<LevelTerrainSettings />, { container });

    expect(getByRole("slider", { name: "Scale" })).toBeDisabled();

    act(() =>
      renderSettings.setRendererOverrides({
        terrain: { ...DEFAULT_RENDER_TERRAIN_SETTINGS, tiling: ERenderTerrainTiling.VARIED },
      })
    );

    expect(getByRole("slider", { name: "Scale" })).toBeEnabled();
    expect(getByRole("slider", { name: "Scale" })).toHaveAttribute("aria-valuetext", "2.00×");
    expect(getByRole("slider", { name: "Rotation" })).toHaveAttribute("aria-valuetext", "±0°");
    expect(getByRole("slider", { name: "Blend sharpness" })).toHaveAttribute("aria-valuetext", "7.0");
  });
});
