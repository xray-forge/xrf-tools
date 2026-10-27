import { beforeEach, describe, expect, it } from "@jest/globals";
import { within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ERenderReflectionMode, ERenderTraceSize, ERenderWaterMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelReflectionsAction } from "@/applications/level-viewer/components/toolbar/lighting/LevelReflectionsAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import {
  DEFAULT_RENDER_REFLECTION_SETTINGS,
  DEFAULT_RENDER_WATER_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";

/**
 * @param reflections - The reflections the settings draw every viewport with.
 * @param waterMode - How the settings draw the water.
 * @returns The rendered action and the services behind it.
 */
function renderAction(
  reflections: TRenderReflectionSettings = DEFAULT_RENDER_REFLECTION_SETTINGS,
  waterMode: ERenderWaterMode = ERenderWaterMode.ENHANCED
): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({
    reflections,
    water: { ...DEFAULT_RENDER_WATER_SETTINGS, mode: waterMode },
  });

  return renderLevelView(<LevelReflectionsAction />, descriptor);
}

/**
 * @param rendered - The rendered action.
 */
async function open(rendered: TLevelViewRenderResult): Promise<void> {
  await userEvent.click(rendered.getByRole("button", { name: "Reflections" }));
  await rendered.findByRole("dialog", { name: "Reflections" });
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelReflectionsAction", () => {
  it("turns the surfaces' reflections on, their parameters settable only then, and back to the preset", async () => {
    const rendered: TLevelViewRenderResult = renderAction();
    const { getByRole, view } = rendered;

    await open(rendered);

    expect(getByRole("slider", { name: "Steps" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Ray budgets" })).toBeDisabled();

    await userEvent.click(getByRole("checkbox", { name: "Surfaces" }));

    expect(view.features.reflections).toEqual({ mode: ERenderReflectionMode.ENHANCED });
    expect(getByRole("slider", { name: "Steps" })).toBeEnabled();
    expect(getByRole("slider", { name: "Steps" })).toHaveAttribute("aria-valuetext", "96");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "150 m");
    expect(getByRole("checkbox", { name: "Ray budgets" })).toBeChecked();
    expect(getByRole("checkbox", { name: "Trace dry surfaces" })).not.toBeChecked();

    await userEvent.click(within(getByRole("group", { name: "Trace size" })).getByRole("button", { name: "Full" }));

    expect(view.features.reflections).toEqual({
      mode: ERenderReflectionMode.ENHANCED,
      traceSize: ERenderTraceSize.FULL,
    });

    await userEvent.click(getByRole("button", { name: "Reset the reflections to the preset" }));

    expect(view.features.reflections).toEqual({});
  });

  it("greys the trace size out at full under ray reconstruction", async () => {
    const rendered: TLevelViewRenderResult = renderAction({
      ...DEFAULT_RENDER_REFLECTION_SETTINGS,
      mode: ERenderReflectionMode.ENHANCED,
    });

    noteLevelFrame(rendered.render, { adapter: "GPU", hasRayQueries: true, isRayReconstructed: true });
    await open(rendered);

    const full: HTMLElement = within(rendered.getByRole("group", { name: "Trace size" })).getByRole("button", {
      name: "Full",
    });

    expect(full).toBeDisabled();
    expect(full).toHaveAttribute("aria-pressed", "true");
  });

  it("turns the water's screen reflection off, and greys it out over the game's water", async () => {
    const enhanced: TLevelViewRenderResult = renderAction();

    await open(enhanced);
    await userEvent.click(enhanced.getByRole("checkbox", { name: "Water" }));

    expect(enhanced.view.features.reflections).toEqual({ isWaterReflected: false });

    enhanced.unmount();

    const engine: TLevelViewRenderResult = renderAction(DEFAULT_RENDER_REFLECTION_SETTINGS, ERenderWaterMode.ENGINE);

    await open(engine);

    expect(engine.getByRole("checkbox", { name: "Water" })).toBeDisabled();
  });

  it("offers how often to re-trace once tracing what the screen misses is on", async () => {
    const rendered: TLevelViewRenderResult = renderAction({ ...DEFAULT_RENDER_REFLECTION_SETTINGS, isTraced: true });

    noteLevelFrame(rendered.render, { adapter: "GPU", hasRayQueries: true });
    await open(rendered);

    expect(rendered.getByRole("checkbox", { name: "Ray tracing" })).toBeChecked();
    expect(rendered.getByRole("slider", { name: "Re-trace" })).toHaveAttribute("aria-valuetext", "Every 2 frames");
  });

  it("greys out tracing what the screen misses where the GPU traces no rays", async () => {
    const rendered: TLevelViewRenderResult = renderAction({ ...DEFAULT_RENDER_REFLECTION_SETTINGS, isTraced: true });

    noteLevelFrame(rendered.render, { adapter: "GPU", hasRayQueries: false });
    await open(rendered);

    const traced: HTMLElement = rendered.getByRole("checkbox", { name: "Ray tracing" });

    expect(traced).toBeDisabled();
    expect(traced).not.toBeChecked();
    expect(traced).toHaveAccessibleDescription("Needs a GPU that traces rays");
    expect(rendered.getByRole("slider", { name: "Re-trace" })).toBeDisabled();
  });
});
