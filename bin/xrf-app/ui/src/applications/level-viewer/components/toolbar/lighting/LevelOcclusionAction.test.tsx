import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderAmbientOcclusionMethod, ERenderIndirectLightMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelOcclusionAction } from "@/applications/level-viewer/components/toolbar/lighting/LevelOcclusionAction";
import {
  DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param setup - What the case sets on the services before it draws.
 * @returns The occlusion's action, its popover open, and the services behind it, from the default occlusion.
 */
async function renderOpenAction(
  setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {}
): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  // The occlusion and the indirect light start from their defaults, whatever the preset draws them with.
  descriptor.renderSettings.setRendererOverrides({
    ambientOcclusion: DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
    indirectLight: DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
  });
  setup(descriptor);

  const rendered: TLevelViewRenderResult = renderLevelView(<LevelOcclusionAction />, descriptor);

  await userEvent.click(rendered.getByRole("button", { name: "Ambient light" }));
  await rendered.findByRole("dialog", { name: "Ambient light" });

  return rendered;
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelOcclusionAction", () => {
  it("sets the screen occlusion back to the settings and the baked one back to whole", async () => {
    const { getByRole, view } = await renderOpenAction(({ view }) => {
      view.setFeatures({ ...view.features, ambientOcclusion: { radius: 2 } });
      view.setHemiStrength(0.5);
    });

    await userEvent.click(getByRole("button", { name: "Reset the ambient occlusion to the preset" }));
    await userEvent.click(getByRole("button", { name: "Reset to the game's" }));

    expect(view.features.ambientOcclusion).toEqual({});
    expect(view.hemiStrength).toBe(1);
  });

  it("switches the view to VBAO, offering its strengths only then", async () => {
    const { getByRole, queryByRole, view } = await renderOpenAction();

    expect(queryByRole("slider", { name: "Thickness" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "VBAO" }));

    expect(view.features.ambientOcclusion).toEqual({ method: ERenderAmbientOcclusionMethod.VBAO });
    expect(getByRole("slider", { name: "Thickness" })).toHaveAttribute("aria-valuetext", "0.25 m");
    expect(getByRole("slider", { name: "Accumulation" })).toHaveAttribute("aria-valuetext", "8 frames");
    expect(getByRole("button", { hidden: true, name: "Ambient light" })).toHaveAccessibleDescription(
      "VBAO occlusion over 1.00 m, 1 × 8 steps, baked at 100%"
    );
  });

  it("offers RTAO only on a GPU that traces rays, saying VBAO searches in its place elsewhere", async () => {
    const rendered = await renderOpenAction(({ renderSettings }) =>
      renderSettings.setRendererOverrides({ ambientOcclusion: { method: ERenderAmbientOcclusionMethod.RTAO } })
    );

    expect(rendered.getByRole("button", { name: "RTAO" })).toBeEnabled();
    expect(rendered.getByRole("slider", { name: "Accumulation" })).toBeInTheDocument();
    expect(rendered.queryByRole("slider", { name: "Thickness" })).toBeNull();

    noteLevelFrame(rendered.render, { adapter: "Test GPU", hasRayQueries: false });

    expect(rendered.getByRole("button", { name: "RTAO" })).toBeDisabled();
    expect(rendered.getByText("RTAO needs a GPU that traces rays: VBAO searches in its place.")).toBeInTheDocument();
  });

  it("turns the view's indirect light to enhanced, offering its intensity only then, and back to the settings", async () => {
    const { getByRole, queryByRole, view } = await renderOpenAction();

    expect(getByRole("checkbox", { name: "Indirect light" })).not.toBeChecked();
    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Indirect light" }));

    expect(view.features.indirectLight).toEqual({ mode: ERenderIndirectLightMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Indirect light" })).toBeChecked();
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "100%");
    expect(getByRole("button", { hidden: true, name: "Ambient light" })).toHaveAccessibleDescription(
      "GTAO occlusion over 1.00 m, 3 × 3 steps, indirect light, baked at 100%"
    );

    await userEvent.click(getByRole("button", { name: "Reset the indirect light to the preset" }));

    expect(view.features.indirectLight).toEqual({});
  });
});
