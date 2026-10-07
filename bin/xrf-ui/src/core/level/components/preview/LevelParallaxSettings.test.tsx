import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { LevelParallaxSettings } from "@/core/level/components/preview/LevelParallaxSettings";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_RENDER_PARALLAX_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

type TChange = (features: ILevelFeatureOptions) => void;

describe("LevelParallaxSettings", () => {
  it("sets the view's own mode over the settings, and gives it back to them", async () => {
    const onChange = jest.fn<TChange>();
    const { getByRole } = renderWithProviders(
      <LevelParallaxSettings
        value={DEFAULT_RENDER_PARALLAX_SETTINGS}
        features={mockLevelFeatureOptions()}
        onChange={onChange}
      />
    );

    await userEvent.click(getByRole("button", { name: "Enhanced" }));
    await userEvent.click(getByRole("button", { name: "Back to the settings" }));

    expect(onChange.mock.calls.map(([features]) => features.parallax)).toEqual([
      { mode: ERenderParallaxMode.ENHANCED },
      {},
    ]);
  });

  it("shows the enhanced parallax's strengths only while it is enhanced", () => {
    const { queryByRole, rerender } = renderWithProviders(
      <LevelParallaxSettings
        value={DEFAULT_RENDER_PARALLAX_SETTINGS}
        features={mockLevelFeatureOptions()}
        onChange={() => {}}
      />
    );

    expect(queryByRole("slider", { name: "Steps" })).toBeNull();

    rerender(
      <LevelParallaxSettings
        value={{ ...DEFAULT_RENDER_PARALLAX_SETTINGS, mode: ERenderParallaxMode.ENHANCED }}
        features={mockLevelFeatureOptions()}
        onChange={() => {}}
      />
    );

    expect(queryByRole("slider", { name: "Steps" })).toHaveAttribute("aria-valuetext", "16");
    expect(queryByRole("slider", { name: "Range" })).toHaveAttribute("aria-valuetext", "60 m");
  });
});
