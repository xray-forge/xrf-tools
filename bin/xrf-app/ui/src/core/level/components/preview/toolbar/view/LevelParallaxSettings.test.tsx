import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderParallaxMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelParallaxSettings } from "@/core/level/components/preview/toolbar/view/LevelParallaxSettings";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelParallaxSettings", () => {
  it("sets the view's own mode over the settings, the enhanced strengths settable only then, and gives it back", async () => {
    const { container, renderSettings, view } = mockLevelViewContainer();

    // The settings draw the engine's, so the view's own choice is what turns it enhanced.
    renderSettings.setRendererOverrides({ parallax: { mode: ERenderParallaxMode.ENGINE } });

    const { getByRole } = renderWithProviders(<LevelParallaxSettings />, { container });

    expect(getByRole("slider", { name: "Steps" })).toBeDisabled();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.parallax).toEqual({ mode: ERenderParallaxMode.ENHANCED });
    expect(getByRole("slider", { name: "Steps" })).toBeEnabled();
    expect(getByRole("slider", { name: "Steps" })).toHaveAttribute("aria-valuetext", "8");
    expect(getByRole("slider", { name: "Range" })).toHaveAttribute("aria-valuetext", "60 m");

    await userEvent.click(getByRole("button", { name: "Reset to the preset" }));

    expect(view.features.parallax).toEqual({});
    expect(getByRole("slider", { name: "Steps" })).toBeDisabled();
  });
});
