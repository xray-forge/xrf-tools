import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import {
  ERenderAntialiasing,
  ERenderDlssAvailability,
  ERenderTerrainTiling,
  RenderAntialiasing,
} from "@xrf/ipc/types/xrf-renderer";

import { LevelShadingAction } from "@/core/level/components/preview/toolbar/view/LevelShadingAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  noteLevelFrame,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import {
  DEFAULT_RENDER_PARALLAX_SETTINGS,
  DEFAULT_RENDER_TERRAIN_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";

/**
 * @param setup - What the case sets on the services before it draws.
 * @param antialiasing - What the settings smooth the frame with.
 * @returns The shading's action and the services behind it, the parallax and the terrain's tiling as the engine draws
 *   them.
 */
function renderAction(
  setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {},
  antialiasing: RenderAntialiasing = ERenderAntialiasing.SMAA
): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({
    antialiasing,
    parallax: DEFAULT_RENDER_PARALLAX_SETTINGS,
    terrain: DEFAULT_RENDER_TERRAIN_SETTINGS,
  });
  setup(descriptor);

  return renderLevelView(<LevelShadingAction />, descriptor);
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelShadingAction", () => {
  it("opens the shading on a click, and turns each surface switch over by its own checkbox", async () => {
    const { getByRole, findByRole, view } = renderAction();

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

  it("varies the terrain's tiling for the view and turns it back to the engine's grid", async () => {
    const { getByRole, findByRole, view } = renderAction();

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });

    expect(getByRole("checkbox", { name: "Varied terrain" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Varied terrain" }));

    expect(view.features.terrain).toEqual({ tiling: ERenderTerrainTiling.VARIED });
    expect(getByRole("checkbox", { name: "Varied terrain" })).toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Varied terrain" }));

    expect(view.features.terrain).toEqual({ tiling: ERenderTerrainTiling.ENGINE });
  });

  // Clay and the shader colours replace the textures checkbox: what the surfaces show is one choice with the targets.
  it("shows the frame as clay, by shader, or as one of its targets", async () => {
    const { getByRole, findByRole, view } = renderAction(({ view }) => view.setShading(ELevelShading.CLAY));

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Clay");

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });
    await userEvent.click(getByRole("option", { name: "Depth" }));

    expect(view.shading).toBe(ELevelShading.DEPTH);
  });

  it("lists the shadings under what they show, saying what the one chosen is", async () => {
    const { getByRole, findByRole, getByText } = renderAction();

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });

    for (const section of ["Frame", "Surfaces", "Light", "Temporal"]) {
      expect(getByRole("group", { name: section })).toBeTruthy();
    }

    expect(getByRole("option", { name: "Depth" }).closest("[role=group]")).toBe(
      getByRole("group", { name: "Surfaces" })
    );
    expect(getByText("The finished frame, every pass applied.")).toBeTruthy();

    await userEvent.click(getByRole("option", { name: "Baked sun" }));

    expect(getByText("How much of the sun each surface gets, as the level compiler baked it.")).toBeTruthy();
  });

  // FSR draws its own targets alone; under any other antialiasing they would show a resolve the frame does not use.
  it("offers FSR's views only while FSR draws the frame", async () => {
    const smaa: TLevelViewRenderResult = renderAction();

    await userEvent.click(smaa.getByRole("button", { name: "Shading" }));
    await smaa.findByRole("dialog", { name: "Shading" });

    expect(smaa.getByRole("option", { name: "Motion" })).toBeTruthy();
    expect(smaa.queryByRole("option", { name: "FSR masks" })).toBeNull();

    smaa.unmount();

    const fsr: TLevelViewRenderResult = renderAction(() => {}, ERenderAntialiasing.FSR);

    await userEvent.click(fsr.getByRole("button", { name: "Shading" }));
    await fsr.findByRole("dialog", { name: "Shading" });
    await userEvent.click(fsr.getByRole("option", { name: "FSR masks" }));

    expect(fsr.view.shading).toBe(ELevelShading.FSR_MASKS);
  });

  it("offers FSR's views where FSR draws in place of DLSS", async () => {
    const { getByRole, findByRole, render } = renderAction(() => {}, ERenderAntialiasing.DLSS);

    noteLevelFrame(render, { adapter: "Test GPU", dlss: ERenderDlssAvailability.UNSUPPORTED });

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });

    expect(getByRole("option", { name: "FSR locks" })).toBeTruthy();
  });

  // The choice is kept, so it comes back with FSR, but the viewport is never left showing a target nothing lists.
  it("shows the final frame for an FSR view once FSR stops drawing, and the view again once it draws", () => {
    const { getByRole, render, renderSettings, view } = renderAction(
      ({ view }) => view.setShading(ELevelShading.FSR_LOCKS),
      ERenderAntialiasing.FSR
    );

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("FSR locks");

    act(() => renderSettings.setRendererOverrides({ antialiasing: ERenderAntialiasing.TAA }));

    expect(view.shading).toBe(ELevelShading.FSR_LOCKS);
    expect(render.shading).toBe(ELevelShading.FINAL);
    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Final frame");

    act(() => renderSettings.setRendererOverrides({ antialiasing: ERenderAntialiasing.FSR }));

    expect(render.shading).toBe(ELevelShading.FSR_LOCKS);
  });
});
