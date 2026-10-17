import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { LevelParallaxSettings } from "@/core/level/components/preview/toolbar/view/LevelParallaxSettings";
import { clearLocalStorage } from "@/core/storage";
import { mockLevelViewContainer } from "@/fixtures/utils/level-container";
import { renderWithProviders } from "@/fixtures/utils/render";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelParallaxSettings", () => {
  it("sets the view's own mode over the settings, showing the enhanced strengths only then, and gives it back", async () => {
    const { container, renderSettings, view } = mockLevelViewContainer();

    // The settings draw the engine's, so the view's own choice is what turns it enhanced.
    renderSettings.setRendererOverrides({ parallax: { mode: ERenderParallaxMode.ENGINE } });

    const { getByRole, queryByRole } = renderWithProviders(<LevelParallaxSettings />, { container });

    expect(queryByRole("slider", { name: "Steps" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.parallax).toEqual({ mode: ERenderParallaxMode.ENHANCED });
    expect(getByRole("slider", { name: "Steps" })).toHaveAttribute("aria-valuetext", "8");
    expect(getByRole("slider", { name: "Range" })).toHaveAttribute("aria-valuetext", "60 m");

    await userEvent.click(getByRole("button", { name: "Reset to the preset" }));

    expect(view.features.parallax).toEqual({});
    expect(queryByRole("slider", { name: "Steps" })).toBeNull();
  });
});
