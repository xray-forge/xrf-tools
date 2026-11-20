import { afterEach, describe, expect, it } from "@jest/globals";
import { act, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ERenderBloomMode, ERenderDisplayTransform, ERenderLightModel } from "@xrf/ipc/types/xrf-renderer";

import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { ELevelLookSource } from "@/core/level/lib/look/level-look-choice";
import { clearLocalStorage } from "@/core/storage";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LevelLookAction } from "@/tools/level-viewer/components/toolbar/image/LevelLookAction";

function renderAction(): TLevelViewRenderResult {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  return renderLevelView(<LevelLookAction />, descriptor);
}

async function pickSource(rendered: TLevelViewRenderResult, name: string): Promise<void> {
  await userEvent.click(rendered.getByRole("combobox", { name: "Source" }));
  await userEvent.click(rendered.getByRole("option", { name }));
}

afterEach(() => {
  clearLocalStorage();
});

describe("LevelLookAction", () => {
  it("draws with the settings' look, and saves no presets of its own", async () => {
    const rendered: TLevelViewRenderResult = renderAction();
    const { getByRole, findByRole, queryByRole, look } = rendered;

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await pickSource(rendered, "Settings");

    expect(look.choice.source).toBe(ELevelLookSource.SETTINGS);
    expect(queryByRole("textbox", { name: "Preset name" })).not.toBeInTheDocument();
  });

  it("offers XRF's, Anomaly's and OpenXRay's own looks whatever game is open", async () => {
    const rendered: TLevelViewRenderResult = renderAction();
    const { getByRole, findByRole, look } = rendered;

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await pickSource(rendered, "XRF");

    expect(look.choice.source).toBe(ELevelLookSource.XRF);
    expect(look.look.lightScales).toEqual({ ambient: 0.1, hemi: 0.6, sun: 1 });
    expect(look.look.exposure.middleGray).toBe(1.1);

    await pickSource(rendered, "Anomaly");

    expect(look.choice.source).toBe(ELevelLookSource.ANOMALY);
    expect(look.look.lightScales.sun).toBe(2);
    expect(look.look.exposure.middleGray).toBe(1.5);

    await pickSource(rendered, "OpenXRay");

    expect(look.look.lightScales.sun).toBe(1);
    expect(look.look.exposure.middleGray).toBe(1);
  });

  // Trying another look does not throw away the values edited by hand.
  it("keeps the values edited by hand to go back to after another look is picked", async () => {
    const rendered: TLevelViewRenderResult = renderAction();
    const { getByRole, findByRole, look } = rendered;

    act(() => look.edit({ ...look.look, lightScales: { ...look.look.lightScales, sun: 3 } }));

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });
    await pickSource(rendered, "Settings");
    await pickSource(rendered, "Edited by hand");

    expect(look.choice.source).toBe(ELevelLookSource.CUSTOM);
    expect(look.look.lightScales.sun).toBe(3);
  });

  // The enhanced bloom's strengths stand in the engine bloom's place while it draws.
  it("draws the enhanced bloom on asking, its strengths in place of the engine bloom's", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    // The settings draw the engine bloom, so the view's own choice is what turns it enhanced.
    descriptor.renderSettings.setRendererOverrides({ enhancedBloom: { mode: ERenderBloomMode.ENGINE } });

    const { getByRole, findByRole } = renderWithProviders(<LevelLookAction />, {
      container: descriptor.container,
    });

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });

    expect(getByRole("checkbox", { name: "Engine bloom" })).toBeEnabled();
    expect(getByRole("slider", { name: "Vibrance" })).toBeDisabled();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(descriptor.view.features.enhancedBloom).toEqual({ mode: ERenderBloomMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Engine bloom" })).toBeDisabled();
    expect(getByRole("slider", { name: "Radius" })).toBeDisabled();
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

  it("offers the physical light's display transform, but off under the engine light", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelLookAction />, { container: descriptor.container });

    await userEvent.click(getByRole("button", { name: "Look" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Look" });

    for (const section of ["Light", "Exposure", "Display", "Bloom", "Image", "Debanding"]) {
      expect(within(dialog).getByRole("group", { name: section })).toBeInTheDocument();
    }

    const display: HTMLElement = within(dialog).getByRole("group", { name: "Display" });
    const exposure: HTMLElement = within(dialog).getByRole("group", { name: "Exposure" });

    expect(within(display).getByRole("button", { name: "AgX" })).toBeDisabled();
    expect(within(exposure).getByRole("slider", { name: "Compensation" })).toBeDisabled();
    expect(within(exposure).getByRole("slider", { name: "Middle gray" })).toBeEnabled();
  });

  it("shows the physical light through the view's own display transform", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelLookAction />, { container: descriptor.container });

    await userEvent.click(getByRole("button", { name: "Look" }));

    const dialog: HTMLElement = await findByRole("dialog", { name: "Look" });
    const light: HTMLElement = within(dialog).getByRole("group", { name: "Light" });
    const exposure: HTMLElement = within(dialog).getByRole("group", { name: "Exposure" });

    await userEvent.click(
      within(within(light).getByRole("group", { name: "Model" })).getByRole("button", { name: "Physical" })
    );
    await userEvent.click(within(dialog).getByRole("button", { name: "AgX" }));

    expect(descriptor.view.features.displayTransform).toBe(ERenderDisplayTransform.AGX);
    expect(descriptor.renderSettings.rendererFeatures.displayTransform).toBe(ERenderDisplayTransform.ENGINE);
    expect(within(exposure).getByRole("slider", { name: "Compensation" })).toBeEnabled();
    // The engine meter's values, which the physical light's histogram does not read; the hemisphere's scale still
    // scales the sky's irradiance, and the game's flat ambient stays at the authored levels, off at the physical ones.
    expect(within(exposure).getByRole("slider", { name: "Middle gray" })).toBeDisabled();
    expect(within(light).getByRole("slider", { name: "Hemisphere" })).toBeEnabled();
    expect(within(light).getByRole("slider", { name: "Ambient" })).toBeEnabled();
    expect(within(light).getByRole("slider", { name: "Sun tint" })).toBeDisabled();
  });

  it("lights the physical levels by the world's sun, tinted by the weather's, with no flat ambient", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const { getByRole, findByRole } = renderWithProviders(<LevelLookAction />, { container: descriptor.container });

    await userEvent.click(getByRole("button", { name: "Look" }));

    const light: HTMLElement = within(await findByRole("dialog", { name: "Look" })).getByRole("group", {
      name: "Light",
    });

    await userEvent.click(
      within(within(light).getByRole("group", { name: "Model" })).getByRole("button", { name: "Physical" })
    );
    await userEvent.click(
      within(within(light).getByRole("group", { name: "Levels" })).getByRole("button", { name: "Physical" })
    );

    expect(within(light).getByRole("slider", { name: "Ambient" })).toBeDisabled();
    expect(within(light).getByRole("slider", { name: "Sun tint" })).toBeEnabled();
  });
});
