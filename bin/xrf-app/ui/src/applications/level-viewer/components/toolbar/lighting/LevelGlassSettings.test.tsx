import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderGlassMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelGlassSettings } from "@/applications/level-viewer/components/toolbar/lighting/LevelGlassSettings";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelGlassSettings", () => {
  it("turns the view's glass to enhanced, offering its refraction only then, and back to the settings", async () => {
    const { container, renderSettings, view } = mockLevelViewContainer();

    // The settings draw the engine's, so the view's own choice is what turns it enhanced.
    renderSettings.setRendererOverrides({ glass: { mode: ERenderGlassMode.ENGINE } });

    const { getByRole, queryByRole } = renderWithProviders(<LevelGlassSettings />, { container });

    expect(queryByRole("slider", { name: "Refraction" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.glass).toEqual({ mode: ERenderGlassMode.ENHANCED });
    expect(getByRole("slider", { name: "Refraction" })).toHaveAttribute("aria-valuetext", "1.00");

    await userEvent.click(getByRole("button", { name: "Reset the glass to the preset" }));

    expect(view.features.glass).toEqual({});
    expect(queryByRole("slider", { name: "Refraction" })).toBeNull();
  });
});
