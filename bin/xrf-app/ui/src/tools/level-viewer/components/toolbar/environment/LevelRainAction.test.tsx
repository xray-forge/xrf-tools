import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { resetMockInvoke } from "@xrf/ipc/testing";
import { ERenderRainDropsMode, ERenderRainMode } from "@xrf/ipc/types/xrf-renderer";

import {
  ILevelViewContainerDescriptor,
  ILevelViewOpening,
  mockLevelViewContainer,
  renderLevelView,
  renderOpenLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_RENDER_RAIN_SETTINGS } from "@/core/render/lib/settings/choice/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import { LevelRainAction } from "@/tools/level-viewer/components/toolbar/environment/LevelRainAction";

/**
 * @param opening - The level and weather to open; none unless given.
 * @returns The rain's action and the services behind it, no puddles drawn.
 */
async function renderAction(opening?: ILevelViewOpening): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ rain: DEFAULT_RENDER_RAIN_SETTINGS });

  return opening
    ? renderOpenLevelView(<LevelRainAction />, descriptor, opening)
    : renderLevelView(<LevelRainAction />, descriptor);
}

beforeEach(() => {
  clearLocalStorage();
});

afterEach(() => {
  resetMockInvoke();
});

describe("LevelRainAction", () => {
  it("strikes with a collection the game has, or none, and switches the thunder apart from the rain", async () => {
    const { getByRole, findByRole, view, weather } = await renderAction({
      weather: {
        thunderboltCollections: [
          { file: "environment\\thunderbolt_collections.ltx", name: "collection_default", thunderbolts: ["a", "b"] },
        ],
      },
    });

    act(() => weather.editManual({ thunderboltCollection: "collection_stancia" }));

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });
    await userEvent.click(getByRole("checkbox", { name: "Thunder" }));

    expect(view.options.isThundering).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isThundering);

    await userEvent.click(getByRole("combobox", { name: "Lightning bolts" }));

    // A collection the game no longer has is kept, so the keyframe still says what it names.
    expect(getByRole("listbox").textContent).toMatch(
      /None.*collection_stancianot in the game.*collection_default2 bolts/
    );

    await userEvent.click(getByRole("option", { name: /collection_default/ }));

    expect(weather.manual?.thunderboltCollection).toBe("collection_default");
  });

  it("gathers puddles on the ground as the rain's last section, their strengths settable only then", async () => {
    const { getByRole, findByRole, queryByRole, view } = await renderAction();

    expect(getByRole("button", { name: "Rain" })).toHaveAccessibleDescription("No rain, no thunder, no puddles");

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });

    expect(getByRole("slider", { name: "Amount" })).toBeDisabled();

    await userEvent.click(getByRole("checkbox", { name: "Puddles" }));

    expect(view.features.rain).toEqual({ mode: ERenderRainMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Puddles" })).toBeChecked();
    expect(getByRole("slider", { name: "Amount" })).toBeEnabled();
    expect(getByRole("slider", { name: "Amount" })).toHaveAttribute("aria-valuetext", "50%");
    expect(getByRole("slider", { name: "Reflectivity" })).toHaveAttribute("aria-valuetext", "40%");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "300 m");
    expect(getByRole("button", { hidden: true, name: "Rain" })).toHaveAccessibleDescription(
      "No rain, no thunder, puddles at 50% of the sites"
    );

    await userEvent.click(getByRole("button", { name: "Puddles: Back to the settings" }));

    expect(view.features.rain).toEqual({});
    expect(queryByRole("button", { name: "Puddles: Back to the settings" })).toBeNull();
  });

  it("gives each section's settings back on their own, the drops' reset keeping the puddles", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ rain: DEFAULT_RENDER_RAIN_SETTINGS });
    descriptor.view.setFeatures({
      ...descriptor.view.features,
      rain: { dropSquare: 20, drops: ERenderRainDropsMode.ENHANCED, mode: ERenderRainMode.ENHANCED },
    });

    const { getByRole, findByRole, queryByRole, view } = renderLevelView(<LevelRainAction />, descriptor);

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });
    await userEvent.click(getByRole("button", { name: "Refracting drops: Back to the settings" }));

    expect(view.features.rain).toEqual({ mode: ERenderRainMode.ENHANCED });
    expect(queryByRole("button", { name: "Refracting drops: Back to the settings" })).toBeNull();
    expect(getByRole("button", { name: "Puddles: Back to the settings" })).toBeInTheDocument();
  });

  it("sets the rain's keys back to the weather from its heading, apart from the thunder's", async () => {
    const { getByRole, findByRole, queryByRole, weather } = await renderAction({ weather: {} });
    const density: number = weather.shown.rainDensity;

    act(() => weather.editManual({ rainDensity: density + 0.5 }));

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });

    expect(queryByRole("button", { name: "Thunder: Reset to the weather" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Rain: Reset to the weather" }));

    expect(weather.manual?.rainDensity).toBe(density);
    expect(queryByRole("button", { name: "Rain: Reset to the weather" })).toBeNull();
  });

  it("bends the scene through the falling drops from the rain's own section, apart from the puddles", async () => {
    const { getByRole, findByRole, view } = await renderAction();

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });

    expect(getByRole("checkbox", { name: "Refracting drops" })).not.toBeChecked();

    await userEvent.click(getByRole("checkbox", { name: "Refracting drops" }));

    expect(view.features.rain).toEqual({ drops: ERenderRainDropsMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Refracting drops" })).toBeChecked();
    expect(getByRole("checkbox", { name: "Puddles" })).not.toBeChecked();
  });
});
