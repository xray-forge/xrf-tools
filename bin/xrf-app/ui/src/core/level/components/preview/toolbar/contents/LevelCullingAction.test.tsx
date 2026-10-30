import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderImpostorMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelCullingAction } from "@/core/level/components/preview/toolbar/contents/LevelCullingAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
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

  it("bakes the level's impostors for every view, their quality and share settable only then", async () => {
    const { getByRole, renderSettings } = await renderOpenAction(({ renderSettings }) =>
      renderSettings.setRendererOverrides({ lod: { impostorMode: ERenderImpostorMode.ENGINE, isImpostors: true } })
    );

    expect(getByRole("button", { name: "High" })).toBeDisabled();
    expect(getByRole("slider", { name: "Full quality for" })).toBeDisabled();

    await userEvent.click(getByRole("button", { name: "Baked" }));

    expect(renderSettings.rendererFeatures.lod.impostorMode).toBe(ERenderImpostorMode.BAKED);
    expect(getByRole("button", { name: "High" })).toBeEnabled();
    expect(getByRole("slider", { name: "Full quality for" })).toHaveAttribute("aria-valuetext", "90% of trees");

    await userEvent.click(getByRole("button", { name: "High" }));

    expect(renderSettings.rendererFeatures.lod.impostorSize).toBe(2048);
  });

  it("turns the opaque depth pass off for the view alone", async () => {
    const { getByRole, view } = await renderOpenAction(({ renderSettings }) =>
      renderSettings.setRendererOverrides({ isDepthPrepassed: true })
    );

    await userEvent.click(getByRole("checkbox", { name: "Opaque depth pass" }));

    expect(view.options.isDepthPrepassed).toBe(false);
  });
});
