import { beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { LevelFogAction } from "@/applications/level-viewer/components/toolbar/environment/LevelFogAction";
import { ERenderFogMode } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_RENDER_FOG_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @returns The fog's action, its popover open, and the services behind it, the fog drawn as the engine does.
 */
async function renderOpenAction(): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ fog: DEFAULT_RENDER_FOG_SETTINGS });

  const rendered: TLevelViewRenderResult = renderLevelView(<LevelFogAction />, descriptor);

  await userEvent.pointer({ keys: "[MouseRight]", target: rendered.getByRole("button", { name: "Fog" }) });
  await rendered.findByRole("dialog", { name: "Fog" });

  return rendered;
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelFogAction", () => {
  it("edits a colour component as typed, once it is let go, leaving the others as held", async () => {
    const { getByRole, weather } = await renderOpenAction();
    const green: HTMLElement = getByRole("textbox", { name: "Fog colour green" });

    await userEvent.clear(green);
    await userEvent.type(green, "1.5");

    expect(weather.manual).toBeNull();

    await userEvent.type(green, "{Enter}");

    expect(weather.manual?.fogColor).toEqual([0.304609, 1.5, 0.367354]);
  });

  it("drops a component typed as no number", async () => {
    const { getByRole, weather } = await renderOpenAction();
    const red: HTMLElement = getByRole("textbox", { name: "Fog colour red" });

    await userEvent.clear(red);
    await userEvent.type(red, "-{Enter}");

    expect(weather.manual).toBeNull();
    expect(red).toHaveValue("0.304609");
  });

  it("fades the distance into the sky's haze on asking, the engine's sky while off", async () => {
    const { getByRole, view } = await renderOpenAction();

    act(() => view.setOption("isSkyHazed", false));

    expect(getByRole("button", { name: "Sky (engine)" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(getByRole("button", { name: "Sky haze" }));

    expect(view.options.isSkyHazed).toBe(true);
  });

  it("draws the fog enhanced on asking, offering its strengths only then, and back to the settings", async () => {
    const { getByRole, queryByRole, view } = await renderOpenAction();

    expect(queryByRole("slider", { name: "Height" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.fog).toEqual({ mode: ERenderFogMode.ENHANCED });
    expect(getByRole("slider", { name: "Height" })).toHaveAttribute("aria-valuetext", "8.0 m");
    expect(getByRole("slider", { name: "Density" })).toHaveAttribute("aria-valuetext", "+40%");
    expect(getByRole("slider", { name: "Scattering" })).toHaveAttribute("aria-valuetext", "0%");

    await userEvent.click(getByRole("button", { name: "Reset to the preset" }));

    expect(view.features.fog).toEqual({});
  });
});
