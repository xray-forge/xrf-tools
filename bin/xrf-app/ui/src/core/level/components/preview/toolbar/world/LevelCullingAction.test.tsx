import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelCullingAction } from "@/core/level/components/preview/toolbar/world/LevelCullingAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { clearLocalStorage } from "@/core/storage";

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
  clearLocalStorage();
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

  it("turns the opaque depth pass off for the view alone", async () => {
    const { getByRole, view } = await renderOpenAction(({ renderSettings }) =>
      renderSettings.setRendererOverrides({ isDepthPrepassed: true })
    );

    await userEvent.click(getByRole("checkbox", { name: "Opaque depth pass" }));

    expect(view.options.isDepthPrepassed).toBe(false);
  });

  it("sets the impostors' distance back to the game's from their heading, only while it is off it", async () => {
    const { getByRole, queryByRole, view } = await renderOpenAction(({ renderSettings, view }) => {
      renderSettings.setRendererOverrides({ lod: { isImpostors: true } });
      view.setLod({ ...DEFAULT_LEVEL_LOD_OPTIONS, distance: 2 });
    });

    await userEvent.click(getByRole("button", { name: "Impostors: Reset to the game's" }));

    expect(view.lod).toEqual(DEFAULT_LEVEL_LOD_OPTIONS);
    expect(queryByRole("button", { name: "Impostors: Reset to the game's" })).toBeNull();
  });
});
