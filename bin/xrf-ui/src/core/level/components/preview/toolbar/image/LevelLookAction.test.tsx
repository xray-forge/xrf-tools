import { afterEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ERenderBloomMode } from "@/core/ipc/types/xrf-renderer";
import { LevelLookAction } from "@/core/level/components/preview/toolbar/image/LevelLookAction";
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
});
