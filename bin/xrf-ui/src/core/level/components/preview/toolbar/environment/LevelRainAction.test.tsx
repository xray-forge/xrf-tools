import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ERenderRainDropsMode, ERenderRainMode } from "@/core/ipc/types/xrf-renderer";
import { LevelRainAction } from "@/core/level/components/preview/toolbar/environment/LevelRainAction";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_RENDER_RAIN_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import { resetMockInvoke } from "@/fixtures/mocks/tauri.mocks";
import {
  ILevelViewContainerDescriptor,
  ILevelViewOpening,
  mockLevelViewContainer,
  renderLevelView,
  renderOpenLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

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

  it("gathers puddles on the ground as the rain's third section, offering their strengths only then", async () => {
    const { getByRole, findByRole, queryByRole, view } = await renderAction();

    expect(getByRole("button", { name: "Rain" })).toHaveAccessibleDescription("No rain, no thunder, no puddles");

    await userEvent.click(getByRole("button", { name: "Rain" }));
    await findByRole("dialog", { name: "Rain" });

    expect(queryByRole("slider", { name: "Puddles" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Puddles" }));

    expect(view.features.rain).toEqual({ mode: ERenderRainMode.ENHANCED });
    expect(getByRole("checkbox", { name: "Puddles" })).toBeChecked();
    expect(getByRole("slider", { name: "Puddles" })).toHaveAttribute("aria-valuetext", "50%");
    expect(getByRole("slider", { name: "Reflectivity" })).toHaveAttribute("aria-valuetext", "40%");
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "300 m");
    expect(getByRole("button", { hidden: true, name: "Rain" })).toHaveAccessibleDescription(
      "No rain, no thunder, puddles at 50% of the sites"
    );

    await userEvent.click(getByRole("button", { name: "Reset to the preset" }));

    expect(view.features.rain).toEqual({});
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
