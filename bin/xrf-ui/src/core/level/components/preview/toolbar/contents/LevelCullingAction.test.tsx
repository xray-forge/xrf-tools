import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelCullingAction } from "@/core/level/components/preview/toolbar/contents/LevelCullingAction";
import {
  DEFAULT_RENDER_GRAPH_SETTINGS,
  SERIAL_RENDER_GRAPH_SETTINGS,
} from "@/core/render/lib/settings/render-graph-settings";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param setup - What the case sets on the services before it draws.
 * @returns The culling's action, its popover open, and the services behind it.
 */
async function renderOpenAction(
  setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {}
): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  setup(descriptor);

  const rendered: TLevelViewRenderResult = renderLevelView(<LevelCullingAction />, descriptor);

  await userEvent.click(rendered.getByRole("button", { name: "Culling" }));
  await rendered.findByRole("dialog", { name: "Culling" });

  return rendered;
}

beforeEach(() => {
  window.localStorage.clear();
});

describe("LevelCullingAction", () => {
  it("says why a culling the settings keep off cannot be turned on", async () => {
    const { getByRole } = await renderOpenAction(({ renderSettings }) =>
      renderSettings.setRendererOverrides({ isOcclusionCulled: false, lod: { isImpostors: true } })
    );

    expect(getByRole("checkbox", { name: "Occlusion culling" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Occlusion culling" })).toHaveAccessibleDescription(
      "Off in Settings, under Rendering"
    );
    expect(getByRole("checkbox", { name: "Impostors" })).toBeChecked();
  });

  it("turns the frame graph's optimizations off one at a time or all at once, to bisect a capture", async () => {
    const { getByRole, renderSettings } = await renderOpenAction();

    await userEvent.click(getByRole("checkbox", { name: "Merge render passes" }));

    expect(renderSettings.graph).toEqual({ ...DEFAULT_RENDER_GRAPH_SETTINGS, isMerging: false });

    // With one off, the frame graph's own switch turns them all back on first.
    await userEvent.click(getByRole("checkbox", { name: "Frame graph" }));

    expect(renderSettings.graph).toEqual(DEFAULT_RENDER_GRAPH_SETTINGS);

    await userEvent.click(getByRole("checkbox", { name: "Frame graph" }));

    expect(renderSettings.graph).toEqual(SERIAL_RENDER_GRAPH_SETTINGS);
  });
});
