import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderFoliageMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelWindAction } from "@/applications/level-viewer/components/toolbar/environment/LevelWindAction";
import { TRenderFoliageSettings } from "@/core/render/lib/settings/render-feature-settings";
import { clearLocalStorage } from "@/core/storage";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/fixtures/utils/level-container";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelWindAction", () => {
  it("moves the foliage by the enhanced flow, its strengths offered only then", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ grass: { foliage: { mode: ERenderFoliageMode.ENGINE } } });

    const { getByRole, findByRole, queryByRole, view } = renderLevelView(<LevelWindAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Wind" }) });
    await findByRole("dialog", { name: "Wind" });

    expect(queryByRole("slider", { name: "Trunk bend" })).not.toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(view.features.grass.foliage?.mode).toBe(ERenderFoliageMode.ENHANCED);
    expect(getByRole("slider", { name: "Trunk bend" })).toBeInTheDocument();
    expect(getByRole("slider", { name: "Grass speed" })).toBeInTheDocument();
    expect(queryByRole("slider", { name: "Light through leaves" })).not.toBeInTheDocument();
  });

  // The light through leaves is the Grass popover's, so the foliage's reset here keeps it.
  it("sets the foliage motion back to the preset, keeping the light through leaves and the grass", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();
    const FOLIAGE: TRenderFoliageSettings = descriptor.renderSettings.rendererFeatures.grass.foliage;

    descriptor.view.setFeatures({
      ...descriptor.view.features,
      grass: {
        density: 0.5,
        foliage: {
          ...FOLIAGE,
          mode: ERenderFoliageMode.ENGINE,
          sssIntensity: FOLIAGE.sssIntensity + 0.25,
          treesBend: FOLIAGE.treesBend + 0.5,
        },
      },
    });

    const { getByRole, findByRole, view } = renderLevelView(<LevelWindAction />, descriptor);

    await userEvent.pointer({ keys: "[MouseRight]", target: getByRole("button", { name: "Wind" }) });
    await findByRole("dialog", { name: "Wind" });
    await userEvent.click(getByRole("button", { name: "Reset the foliage motion to the preset" }));

    expect(view.features.grass).toEqual({
      density: 0.5,
      foliage: { ...FOLIAGE, sssIntensity: FOLIAGE.sssIntensity + 0.25 },
    });
  });
});
