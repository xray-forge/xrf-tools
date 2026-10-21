import { afterEach, describe, expect, it } from "@jest/globals";
import { act, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelLookAction } from "@/applications/level-viewer/components/toolbar/image/LevelLookAction";
import { ERenderBloomMode, ERenderDisplayTransform, ERenderLightModel } from "@/core/ipc/types/xrf-renderer";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderAction(): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  return renderLevelView(<LevelLookAction />, descriptor);
}

afterEach(() => {
  clearLocalStorage();
});

describe("LevelLookAction", () => {
  it("draws with the settings' look, and saves no presets of its own", async () => {
    const { getByRole, findByRole, queryByRole, look } = renderAction();

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await userEvent.click(getByRole("option", { name: "Settings" }));

    expect(look.choice.source).toBe(ELevelLookSource.SETTINGS);
    expect(queryByRole("textbox", { name: "Preset name" })).not.toBeInTheDocument();
  });

  it("offers XRF's, Anomaly's and OpenXRay's own looks whatever game is open", async () => {
    const { getByRole, findByRole, look } = renderAction();

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await userEvent.click(getByRole("option", { name: "XRF" }));

    expect(look.choice.source).toBe(ELevelLookSource.XRF);
    expect(look.look.lightScales).toEqual({ ambient: 0.1, hemi: 0.6, sun: 1 });
    expect(look.look.exposure.middleGray).toBe(1.1);

    await userEvent.click(getByRole("option", { name: "Anomaly" }));

    expect(look.choice.source).toBe(ELevelLookSource.ANOMALY);
    expect(look.look.lightScales.sun).toBe(2);
    expect(look.look.exposure.middleGray).toBe(1.5);

    await userEvent.click(getByRole("option", { name: "OpenXRay" }));

    expect(look.look.lightScales.sun).toBe(1);
    expect(look.look.exposure.middleGray).toBe(1);
  });

  // Trying another look does not throw away the values edited by hand.
  it("keeps the values edited by hand to go back to after another look is picked", async () => {
    const { getByRole, findByRole, look } = renderAction();

    act(() => look.edit({ ...look.look, lightScales: { ...look.look.lightScales, sun: 3 } }));

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await userEvent.click(getByRole("option", { name: "Settings" }));
    await userEvent.click(getByRole("option", { name: "Edited by hand" }));

    expect(look.choice.source).toBe(ELevelLookSource.CUSTOM);
    expect(look.look.lightScales.sun).toBe(3);
  });

  // The enhanced bloom's strengths stand in the engine bloom's place while it draws.
  it("draws the enhanced bloom on asking, its strengths in place of the engine bloom's", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    // The settings draw the engine bloom, so the view's own choice is what turns it enhanced.
    descriptor.renderSettings.setRendererOverrides({ enhancedBloom: { mode: ERenderBloomMode.ENGINE } });

    const { getByRole, findByRole, queryByRole } = renderWithProviders(<LevelLookAction />, {
      container: descriptor.container,
    });

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });

    expect(getByRole("checkbox", { name: "Bloom" })).toBeInTheDocument();
    expect(queryByRole("slider", { name: "Vibrance" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(descriptor.view.features.enhancedBloom).toEqual({ mode: ERenderBloomMode.ENHANCED });
    expect(queryByRole("checkbox", { name: "Bloom" })).toBeNull();
    expect(queryByRole("slider", { name: "Radius" })).toBeNull();
    expect(getByRole("slider", { name: "Threshold" })).toHaveAttribute("aria-valuetext", "5.0");
    expect(getByRole("slider", { name: "Vibrance" })).toHaveAttribute("aria-valuetext", "1.0");
  });

  it("lights the view in linear light on asking, over the settings' engine light", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelLookAction />, { container: descriptor.container });

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });

    expect(descriptor.renderSettings.rendererFeatures.lightModel).toBe(ERenderLightModel.ENGINE);

    await userEvent.click(within(getByRole("group", { name: "Model" })).getByRole("button", { name: "Physical" }));

    expect(descriptor.view.features.lightModel).toBe(ERenderLightModel.PHYSICAL);
    expect(descriptor.renderSettings.rendererFeatures.lightModel).toBe(ERenderLightModel.ENGINE);
  });

  it("shows the physical light through the view's own display transform, offered but off under the engine light", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelLookAction />, { container: descriptor.container });

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });

    for (const section of ["Look", "Light", "Exposure", "Display", "Bloom", "Image"]) {
      expect(getByRole("region", { name: section })).toBeInTheDocument();
    }

    expect(getByRole("button", { name: "AgX" })).toBeDisabled();
    expect(getByRole("slider", { name: "Compensation" })).toBeDisabled();
    expect(getByRole("slider", { name: "Middle gray" })).toBeEnabled();

    await userEvent.click(within(getByRole("group", { name: "Model" })).getByRole("button", { name: "Physical" }));
    await userEvent.click(getByRole("button", { name: "AgX" }));

    expect(descriptor.view.features.displayTransform).toBe(ERenderDisplayTransform.AGX);
    expect(descriptor.renderSettings.rendererFeatures.displayTransform).toBe(ERenderDisplayTransform.ENGINE);
    expect(getByRole("slider", { name: "Compensation" })).toBeEnabled();
    // The engine meter's values, which the physical light's histogram does not read; the hemisphere's scale still
    // scales the sky's irradiance, and the game's flat ambient stays at the authored levels, off at the physical ones.
    expect(getByRole("slider", { name: "Middle gray" })).toBeDisabled();
    expect(getByRole("slider", { name: "Hemisphere" })).toBeEnabled();
    expect(getByRole("slider", { name: "Ambient" })).toBeEnabled();

    expect(getByRole("slider", { name: "Sun tint" })).toBeDisabled();

    await userEvent.click(within(getByRole("group", { name: "Levels" })).getByRole("button", { name: "Physical" }));

    // The real-world levels light by the world's sun, whose hue the weather's may tint, and have no flat ambient.
    expect(getByRole("slider", { name: "Ambient" })).toBeDisabled();
    expect(getByRole("slider", { name: "Sun tint" })).toBeEnabled();
  });
});
