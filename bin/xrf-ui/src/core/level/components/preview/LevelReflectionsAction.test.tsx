import { describe, expect, it, jest } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import {
  ERenderReflectionMode,
  ERenderReflectionQuality,
  ERenderReflectionTraceQuality,
} from "@/core/ipc/types/xrf-renderer";
import { LevelReflectionsAction } from "@/core/level/components/preview/LevelReflectionsAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { DEFAULT_RENDER_REFLECTION_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelReflectionsAction", () => {
  it("turns the view's reflections to enhanced, offering their strengths only then, and back to the settings", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const enhanced: TRenderReflectionSettings = {
      ...DEFAULT_RENDER_REFLECTION_SETTINGS,
      mode: ERenderReflectionMode.ENHANCED,
    };

    function render(value: TRenderReflectionSettings): ReactElement {
      return (
        <LevelReflectionsAction value={value} features={mockLevelFeatureOptions()} isTraceable onChange={onChange} />
      );
    }

    const { getByRole, findByRole, queryByRole, rerender } = renderWithProviders(
      render(DEFAULT_RENDER_REFLECTION_SETTINGS)
    );

    expect(getByRole("button", { name: "Reflections" })).toHaveAccessibleDescription(
      "The sky's cube alone. Right-click for its settings"
    );

    await userEvent.click(getByRole("button", { name: "Reflections" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      reflections: { mode: ERenderReflectionMode.ENHANCED },
    });

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Reflections" }) });
    await findByRole("dialog", { name: "Reflections" });

    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();

    rerender(render(enhanced));

    expect(getByRole("checkbox", { name: "Trace reflections" })).toBeChecked();
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "100%");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "150 m");
    expect(getByRole("slider", { name: "Wet reach" })).toHaveAttribute("aria-valuetext", "300 m");
    expect(getByRole("button", { hidden: true, name: "Reflections" })).toHaveAccessibleDescription(
      "Screen-space reflections, high quality, 100% over 150 m. Right-click for its settings"
    );

    await userEvent.click(getByRole("button", { name: "Ultra" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      reflections: { quality: ERenderReflectionQuality.ULTRA },
    });

    await userEvent.click(getByRole("button", { name: "Back to the settings for the reflections" }));

    expect(onChange).toHaveBeenLastCalledWith({ ...mockLevelFeatureOptions(), reflections: {} });
  });

  it("offers how often to trace once tracing what the screen misses is on", async () => {
    const onChange = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole } = renderWithProviders(
      <LevelReflectionsAction
        value={{ ...DEFAULT_RENDER_REFLECTION_SETTINGS, mode: ERenderReflectionMode.ENHANCED, isTraced: true }}
        features={mockLevelFeatureOptions()}
        isTraceable
        onChange={onChange}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Reflections" }) });
    await findByRole("dialog", { name: "Reflections" });
    await userEvent.click(within(getByRole("group", { name: "Trace quality" })).getByRole("button", { name: "Low" }));

    expect(onChange).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      reflections: { traceQuality: ERenderReflectionTraceQuality.LOW },
    });
  });

  it("greys out tracing what the screen misses where the GPU traces no rays", async () => {
    const { getByRole, findByRole } = renderWithProviders(
      <LevelReflectionsAction
        value={{ ...DEFAULT_RENDER_REFLECTION_SETTINGS, mode: ERenderReflectionMode.ENHANCED, isTraced: true }}
        features={mockLevelFeatureOptions()}
        isTraceable={false}
        onChange={jest.fn()}
      />
    );

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Reflections" }) });
    await findByRole("dialog", { name: "Reflections" });

    const traced: HTMLElement = getByRole("checkbox", { name: "Trace what the screen misses" });

    expect(traced).toBeDisabled();
    expect(traced).not.toBeChecked();
    expect(traced).toHaveAccessibleDescription("Needs a GPU that traces rays");
  });
});
