import { beforeEach, describe, expect, it } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import {
  ERenderReflectionMode,
  ERenderReflectionQuality,
  ERenderReflectionTraceQuality,
} from "@/core/ipc/types/xrf-renderer";
import { LevelReflectionsAction } from "@/core/level/components/preview/toolbar/lighting/LevelReflectionsAction";
import { DEFAULT_RENDER_REFLECTION_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param reflections - The reflections the settings draw every viewport with.
 * @returns The rendered action and the services behind it.
 */
function renderAction(
  reflections: TRenderReflectionSettings = DEFAULT_RENDER_REFLECTION_SETTINGS
): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ reflections });

  return renderLevelView(<LevelReflectionsAction />, descriptor);
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelReflectionsAction", () => {
  it("turns the view's reflections to enhanced, offering their strengths only then, and back to the settings", async () => {
    const { getByRole, findByRole, view } = renderAction();

    expect(getByRole("button", { name: "Reflections" })).toHaveAccessibleDescription(
      "The sky's cube alone. Right-click for its settings"
    );

    await userEvent.click(getByRole("button", { name: "Reflections" }));

    expect(view.features.reflections).toEqual({ mode: ERenderReflectionMode.ENHANCED });

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Reflections" }) });
    await findByRole("dialog", { name: "Reflections" });

    expect(getByRole("checkbox", { name: "Trace reflections" })).toBeChecked();
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "100%");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "150 m");
    expect(getByRole("slider", { name: "Wet reach" })).toHaveAttribute("aria-valuetext", "300 m");
    expect(getByRole("button", { hidden: true, name: "Reflections" })).toHaveAccessibleDescription(
      "Screen-space reflections, high quality, 100% over 150 m. Right-click for its settings"
    );

    await userEvent.click(getByRole("button", { name: "Ultra" }));

    expect(view.features.reflections).toEqual({
      mode: ERenderReflectionMode.ENHANCED,
      quality: ERenderReflectionQuality.ULTRA,
    });

    await userEvent.click(getByRole("button", { name: "Reset the reflections to the preset" }));

    expect(view.features.reflections).toEqual({});
  });

  it("offers how often to trace once tracing what the screen misses is on", async () => {
    const { getByRole, findByRole, view } = renderAction({
      ...DEFAULT_RENDER_REFLECTION_SETTINGS,
      isTraced: true,
      mode: ERenderReflectionMode.ENHANCED,
    });

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Reflections" }) });
    await findByRole("dialog", { name: "Reflections" });
    await userEvent.click(within(getByRole("group", { name: "Trace quality" })).getByRole("button", { name: "Low" }));

    expect(view.features.reflections).toEqual({ traceQuality: ERenderReflectionTraceQuality.LOW });
  });

  it("greys out tracing what the screen misses where the GPU traces no rays", async () => {
    const descriptor = renderAction({
      ...DEFAULT_RENDER_REFLECTION_SETTINGS,
      isTraced: true,
      mode: ERenderReflectionMode.ENHANCED,
    });

    noteLevelFrame(descriptor.render, { adapter: "GPU", hasRayQueries: false });

    await userEvent.pointer({ keys: "[MouseRight]", target: descriptor.getByRole("button", { name: "Reflections" }) });
    await descriptor.findByRole("dialog", { name: "Reflections" });

    const traced: HTMLElement = descriptor.getByRole("checkbox", { name: "Trace what the screen misses" });

    expect(traced).toBeDisabled();
    expect(traced).not.toBeChecked();
    expect(traced).toHaveAccessibleDescription("Needs a GPU that traces rays");
  });
});
