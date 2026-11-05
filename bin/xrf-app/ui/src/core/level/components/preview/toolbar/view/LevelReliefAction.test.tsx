import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderParallaxMode, ERenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";

import { LevelReliefAction } from "@/core/level/components/preview/toolbar/view/LevelReliefAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import {
  DEFAULT_RENDER_PARALLAX_SETTINGS,
  DEFAULT_RENDER_TERRAIN_SETTINGS,
} from "@/core/render/lib/settings/choice/render-feature-defaults";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";

function renderAction(parallax: TRenderParallaxSettings = DEFAULT_RENDER_PARALLAX_SETTINGS): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ parallax, terrain: DEFAULT_RENDER_TERRAIN_SETTINGS });

  return renderLevelView(<LevelReliefAction />, descriptor);
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelReliefAction", () => {
  it("opens the relief on a click, and turns the bumps over by their own checkbox", async () => {
    const { getByRole, findByRole, view } = renderAction();

    expect(getByRole("button", { name: "Relief" })).toHaveAccessibleDescription("Bumps");

    await userEvent.click(getByRole("button", { name: "Relief" }));
    await findByRole("dialog", { name: "Relief" });
    await userEvent.click(getByRole("checkbox", { name: "Bumps" }));

    expect(view.options).toEqual({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isBumped: !DEFAULT_LEVEL_VIEW_OPTIONS.isBumped });
    // The open popover hides the toolbar behind it from assistive technology.
    expect(getByRole("button", { name: "Relief", hidden: true })).toHaveAccessibleDescription("Flat");
  });

  // A switch with no settings to tune is no popover's: the shadings, the wireframe and the wall marks are chosen apart.
  it("offers neither the shadings nor the wireframe and wall marks", async () => {
    const { getByRole, findByRole, queryByRole } = renderAction();

    await userEvent.click(getByRole("button", { name: "Relief" }));
    await findByRole("dialog", { name: "Relief" });

    expect(queryByRole("option", { name: "Clay" })).toBeNull();
    expect(queryByRole("checkbox", { name: "Wireframe" })).toBeNull();
    expect(queryByRole("checkbox", { name: "Wall marks" })).toBeNull();
  });

  it("turns the parallax off while the settings draw it, and offers it disabled while they draw none", async () => {
    const enhanced: TLevelViewRenderResult = renderAction({
      ...DEFAULT_RENDER_PARALLAX_SETTINGS,
      mode: ERenderParallaxMode.ENHANCED,
    });

    await userEvent.click(enhanced.getByRole("button", { name: "Relief" }));
    await enhanced.findByRole("dialog", { name: "Relief" });
    await userEvent.click(enhanced.getByRole("checkbox", { name: "Parallax" }));

    expect(enhanced.view.options.isParallaxed).toBe(false);
    expect(enhanced.getByRole("button", { name: "Relief", hidden: true })).toHaveAccessibleDescription(
      "Bumps, without parallax"
    );

    enhanced.unmount();

    const none: TLevelViewRenderResult = renderAction({
      ...DEFAULT_RENDER_PARALLAX_SETTINGS,
      isSteep: false,
      mode: ERenderParallaxMode.ENGINE,
    });

    await userEvent.click(none.getByRole("button", { name: "Relief" }));
    await none.findByRole("dialog", { name: "Relief" });

    expect(none.getByRole("checkbox", { name: "Parallax" })).toBeDisabled();
  });

  it("offers a section's reset only while the view sets its values, and gives them back to the settings", async () => {
    const { getByRole, findByRole, queryByRole, view } = renderAction();

    await userEvent.click(getByRole("button", { name: "Relief" }));
    await findByRole("dialog", { name: "Relief" });

    expect(queryByRole("button", { name: "Parallax: Back to the settings" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.parallax).toEqual({ mode: ERenderParallaxMode.ENHANCED });

    await userEvent.click(getByRole("button", { name: "Parallax: Back to the settings" }));

    expect(view.features.parallax).toEqual({});
    expect(getByRole("slider", { name: "Steps" })).toBeDisabled();
    expect(queryByRole("button", { name: "Parallax: Back to the settings" })).toBeNull();
  });

  it("varies the terrain's tiling for the view and turns it back to the engine's grid", async () => {
    const { getByRole, findByRole, queryByRole, view } = renderAction();

    await userEvent.click(getByRole("button", { name: "Relief" }));
    await findByRole("dialog", { name: "Relief" });

    expect(getByRole("checkbox", { name: "Varied terrain" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Varied terrain" }));

    expect(view.features.terrain).toEqual({ tiling: ERenderTerrainTiling.VARIED });
    expect(getByRole("button", { name: "Varied terrain: Back to the settings" })).toBeInTheDocument();
    expect(getByRole("checkbox", { name: "Varied terrain" })).toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Varied terrain" }));

    expect(view.features.terrain).toEqual({ tiling: ERenderTerrainTiling.ENGINE });
    // The view's own value is the settings' again, so a reset would change nothing and is not offered.
    expect(queryByRole("button", { name: "Varied terrain: Back to the settings" })).toBeNull();
  });
});
