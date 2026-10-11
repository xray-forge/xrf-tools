import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { ERenderAmbientOcclusionMethod, ERenderIndirectLightMode } from "@/core/ipc/types/xrf-renderer";
import { LevelCullingAction } from "@/core/level/components/preview/LevelCullingAction";
import { LevelOcclusionAction } from "@/core/level/components/preview/LevelOcclusionAction";
import { LevelOverlaysAction } from "@/core/level/components/preview/LevelOverlaysAction";
import { LevelShadingAction } from "@/core/level/components/preview/LevelShadingAction";
import { LevelSpawnAction } from "@/core/level/components/preview/LevelSpawnAction";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import {
  DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
  DEFAULT_RENDER_PARALLAX_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import {
  DEFAULT_RENDER_GRAPH_SETTINGS,
  SERIAL_RENDER_GRAPH_SETTINGS,
} from "@/core/render/lib/settings/render-graph-settings";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param action - The toolbar action to draw.
 * @param setup - What the case sets on the services before it draws.
 * @returns The rendered action and the services behind it.
 */
function renderAction(
  action: ReactElement,
  setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {}
): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  // The occlusion and the parallax start from their defaults, whatever the preset draws them with.
  descriptor.renderSettings.setRendererOverrides({
    ambientOcclusion: DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
    indirectLight: DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
    parallax: DEFAULT_RENDER_PARALLAX_SETTINGS,
  });
  setup(descriptor);

  return renderLevelView(action, descriptor);
}

beforeEach(() => {
  window.localStorage.clear();
});

describe("level toolbar groups", () => {
  it("opens the shading on a click, and turns each surface switch over by its own checkbox", async () => {
    const { getByRole, findByRole, view } = renderAction(<LevelShadingAction />);

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Final frame");

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });
    await userEvent.click(getByRole("checkbox", { name: "Wireframe" }));
    await userEvent.click(getByRole("checkbox", { name: "Bumps" }));
    await userEvent.click(getByRole("checkbox", { name: "Wall marks" }));

    expect(view.options).toEqual({
      ...DEFAULT_LEVEL_VIEW_OPTIONS,
      isBumped: !DEFAULT_LEVEL_VIEW_OPTIONS.isBumped,
      isWallmarked: !DEFAULT_LEVEL_VIEW_OPTIONS.isWallmarked,
      isWireframe: !DEFAULT_LEVEL_VIEW_OPTIONS.isWireframe,
    });
  });

  // Clay and the shader colours replace the textures checkbox: what the surfaces show is one choice with the targets.
  it("shows the frame as clay, by shader, or as one of its targets", async () => {
    const { getByRole, findByRole, view } = renderAction(<LevelShadingAction />, ({ view }) =>
      view.setShading(ELevelShading.CLAY)
    );

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Clay");

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });
    await userEvent.click(getByRole("option", { name: "Depth" }));

    expect(view.shading).toBe(ELevelShading.DEPTH);
  });

  it("names the spawned categories shown, and turns each over by its own checkbox", async () => {
    const { getByRole, findByRole, view } = renderAction(<LevelSpawnAction />, ({ view }) => {
      view.setOption("isSpawnedItems", false);
      view.setOption("isSpawnedWeapons", false);
    });

    expect(getByRole("button", { name: "Spawn" })).toHaveAccessibleDescription("Spawned props, lamps");

    await userEvent.click(getByRole("button", { name: "Spawn" }));
    await findByRole("dialog", { name: "Spawn" });

    expect(getByRole("checkbox", { name: "Items" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Items" }));
    await userEvent.click(getByRole("checkbox", { name: "Lamps" }));

    expect(view.options.isSpawnedItems).toBe(true);
    expect(view.options.isSpawnedLamps).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isSpawnedLamps);
  });

  it("says why a culling the settings keep off cannot be turned on", async () => {
    const { getByRole, findByRole } = renderAction(<LevelCullingAction />, ({ renderSettings }) =>
      renderSettings.setRendererOverrides({ isOcclusionCulled: false, lod: { isImpostors: true } })
    );

    await userEvent.click(getByRole("button", { name: "Culling" }));
    await findByRole("dialog", { name: "Culling" });

    expect(getByRole("checkbox", { name: "Occlusion culling" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Occlusion culling" })).toHaveAccessibleDescription(
      "Off in Settings, under Rendering"
    );
    expect(getByRole("checkbox", { name: "Impostors" })).toBeChecked();
  });

  it("sets the screen occlusion back to the settings and the baked one back to whole", async () => {
    const { getByRole, findByRole, view } = renderAction(<LevelOcclusionAction />, ({ view }) => {
      view.setFeatures({ ...view.features, ambientOcclusion: { radius: 2 } });
      view.setHemiStrength(0.5);
    });

    await userEvent.click(getByRole("button", { name: "Occlusion" }));
    await findByRole("dialog", { name: "Occlusion" });
    await userEvent.click(getByRole("button", { name: "Back to the settings" }));
    await userEvent.click(getByRole("button", { name: "Back to the whole occlusion" }));

    expect(view.features.ambientOcclusion).toEqual({});
    expect(view.hemiStrength).toBe(1);
  });

  it("switches the view to VBAO, offering its strengths only then", async () => {
    const { getByRole, findByRole, queryByRole, view } = renderAction(<LevelOcclusionAction />);

    await userEvent.click(getByRole("button", { name: "Occlusion" }));
    await findByRole("dialog", { name: "Occlusion" });

    expect(queryByRole("slider", { name: "Thickness" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "VBAO" }));

    expect(view.features.ambientOcclusion).toEqual({ method: ERenderAmbientOcclusionMethod.VBAO });
    expect(getByRole("slider", { name: "Thickness" })).toHaveAttribute("aria-valuetext", "0.25 m");
    expect(getByRole("slider", { name: "Accumulation" })).toHaveAttribute("aria-valuetext", "8 frames");
    expect(getByRole("button", { hidden: true, name: "Occlusion" })).toHaveAccessibleDescription(
      "VBAO occlusion over 1.00 m, high quality, baked at 100%"
    );
  });

  it("offers RTAO only on a GPU that traces rays, saying VBAO searches in its place elsewhere", async () => {
    const descriptor = renderAction(<LevelOcclusionAction />, ({ renderSettings }) =>
      renderSettings.setRendererOverrides({ ambientOcclusion: { method: ERenderAmbientOcclusionMethod.RTAO } })
    );

    await userEvent.click(descriptor.getByRole("button", { name: "Occlusion" }));
    await descriptor.findByRole("dialog", { name: "Occlusion" });

    expect(descriptor.getByRole("button", { name: "RTAO" })).toBeEnabled();
    expect(descriptor.getByRole("slider", { name: "Accumulation" })).toBeInTheDocument();
    expect(descriptor.queryByRole("slider", { name: "Thickness" })).toBeNull();

    noteLevelFrame(descriptor.render, { adapter: "Test GPU", hasRayQueries: false });

    expect(descriptor.getByRole("button", { name: "RTAO" })).toBeDisabled();
    expect(descriptor.getByText("RTAO needs a GPU that traces rays: VBAO searches in its place.")).toBeInTheDocument();
  });

  it("turns the view's indirect light to enhanced, offering its intensity only then, and back to the settings", async () => {
    const { getByRole, findByRole, queryByRole, view } = renderAction(<LevelOcclusionAction />);

    await userEvent.click(getByRole("button", { name: "Occlusion" }));
    await findByRole("dialog", { name: "Occlusion" });

    expect(getByRole("checkbox", { name: "Indirect light" })).not.toBeChecked();
    expect(queryByRole("slider", { name: "Intensity" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Indirect light" }));

    expect(view.features.indirectLight).toEqual({ mode: ERenderIndirectLightMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Indirect light" })).toBeChecked();
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "100%");
    expect(getByRole("button", { hidden: true, name: "Occlusion" })).toHaveAccessibleDescription(
      "GTAO occlusion over 1.00 m, high quality, indirect light, baked at 100%"
    );

    await userEvent.click(getByRole("button", { name: "Back to the settings for the indirect light" }));

    expect(view.features.indirectLight).toEqual({});
  });

  it("names the overlays shown, and times the passes from the readouts", async () => {
    const { getByRole, findByRole } = renderAction(<LevelOverlaysAction />, ({ view }) =>
      view.setOption("isAxesVisible", true)
    );

    expect(getByRole("button", { name: "Overlays" })).toHaveAccessibleDescription("Showing axes, readouts");

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Grid" })).not.toBeChecked();
  });

  it("turns the frame graph's optimizations off one at a time or all at once from the culling, to bisect a capture", async () => {
    const { getByRole, findByRole, renderSettings } = renderAction(<LevelCullingAction />);

    await userEvent.click(getByRole("button", { name: "Culling" }));
    await findByRole("dialog", { name: "Culling" });
    await userEvent.click(getByRole("checkbox", { name: "Merge render passes" }));

    expect(renderSettings.graph).toEqual({ ...DEFAULT_RENDER_GRAPH_SETTINGS, isMerging: false });

    // With one off, the frame graph's own switch turns them all back on first.
    await userEvent.click(getByRole("checkbox", { name: "Frame graph" }));

    expect(renderSettings.graph).toEqual(DEFAULT_RENDER_GRAPH_SETTINGS);

    await userEvent.click(getByRole("checkbox", { name: "Frame graph" }));

    expect(renderSettings.graph).toEqual(SERIAL_RENDER_GRAPH_SETTINGS);
  });

  // The marker says where the level's sunlight comes from, which the weather's own sun in the sky already shows by day.
  it("marks the sun among the overlays, off unless asked for", async () => {
    const { getByRole, findByRole, view } = renderAction(<LevelOverlaysAction />);

    await userEvent.click(getByRole("button", { name: "Overlays" }));
    await findByRole("dialog", { name: "Overlays" });

    expect(getByRole("checkbox", { name: "Sun" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Sun" }));

    expect(view.options.isSunMarked).toBe(true);
  });
});
