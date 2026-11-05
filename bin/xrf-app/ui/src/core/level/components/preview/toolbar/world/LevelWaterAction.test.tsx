import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderWaterMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelWaterAction } from "@/core/level/components/preview/toolbar/world/LevelWaterAction";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import {
  DEFAULT_RENDER_REFLECTION_SETTINGS,
  DEFAULT_RENDER_WATER_SETTINGS,
} from "@/core/render/lib/settings/choice/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";

/**
 * @param setup - What the case sets on the services before it draws.
 * @returns The water's action, its popover open, and the services behind it, the engine's water drawn.
 */
async function renderOpenAction(
  setup: (descriptor: ILevelViewContainerDescriptor) => void = () => {}
): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ water: DEFAULT_RENDER_WATER_SETTINGS });
  setup(descriptor);

  const rendered: TLevelViewRenderResult = renderLevelView(<LevelWaterAction />, descriptor);

  await userEvent.pointer({ keys: "[MouseRight]", target: rendered.getByRole("button", { name: "Water" }) });
  await rendered.findByRole("dialog", { name: "Water" });

  return rendered;
}

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelWaterAction", () => {
  it("draws the water enhanced on asking, its values settable only then, and back to the settings", async () => {
    const { getByRole, view } = await renderOpenAction();

    expect(getByRole("slider", { name: "Refraction" })).toBeDisabled();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.water).toEqual({ mode: ERenderWaterMode.ENHANCED });
    expect(getByRole("slider", { name: "Refraction" })).toBeEnabled();

    await userEvent.click(getByRole("button", { name: "Back to the settings" }));

    expect(view.features.water).toEqual({});
  });

  it("says what the enhanced water reflects, by the reflections' settings", async () => {
    const { getByText } = await renderOpenAction(({ renderSettings }) =>
      renderSettings.setRendererOverrides({
        reflections: { ...DEFAULT_RENDER_REFLECTION_SETTINGS, isWaterReflected: false },
        water: { ...DEFAULT_RENDER_WATER_SETTINGS, mode: ERenderWaterMode.ENHANCED },
      })
    );

    expect(getByText("It reflects the sky's cube alone: Reflections, Water reflects the screen.")).toBeInTheDocument();
  });
});
