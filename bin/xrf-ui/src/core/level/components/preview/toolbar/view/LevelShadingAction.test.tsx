import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelShadingAction } from "@/core/level/components/preview/toolbar/view/LevelShadingAction";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_RENDER_PARALLAX_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param setup - What the case sets on the services before it draws.
 * @returns The shading's action and the services behind it, the parallax drawn as the engine does.
 */
function renderAction(setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {}): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ parallax: DEFAULT_RENDER_PARALLAX_SETTINGS });
  setup(descriptor);

  return renderLevelView(<LevelShadingAction />, descriptor);
}

beforeEach(() => {
  window.localStorage.clear();
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

  // Clay and the shader colours replace the textures checkbox: what the surfaces show is one choice with the targets.
  it("shows the frame as clay, by shader, or as one of its targets", async () => {
    const { getByRole, findByRole, view } = renderAction(({ view }) => view.setShading(ELevelShading.CLAY));

    expect(getByRole("button", { name: "Shading" })).toHaveAccessibleDescription("Clay");

    await userEvent.click(getByRole("button", { name: "Shading" }));
    await findByRole("dialog", { name: "Shading" });
    await userEvent.click(getByRole("option", { name: "Depth" }));

    expect(view.shading).toBe(ELevelShading.DEPTH);
  });
});
