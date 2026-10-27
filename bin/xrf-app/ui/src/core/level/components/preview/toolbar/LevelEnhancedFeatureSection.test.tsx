import { beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { ERenderDebandingMode, ERenderGlassMode, ERenderVolumetricMode } from "@xrf/ipc/types/xrf-renderer";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import {
  ILevelViewContainerDescriptor,
  mockLevelViewContainer,
  renderLevelView,
} from "@/core/level/fixtures/level-container";
import { explainRenderVolumetricMode } from "@/core/render/lib/features/render-volumetric-choices";
import { DEFAULT_RENDER_DEBANDING_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";

beforeEach(() => {
  clearLocalStorage();
});

describe("LevelEnhancedFeatureSection", () => {
  it("shows the enhanced feature's values disabled under the engine's, and sets them once it is drawn", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    // The settings draw the engine's, so the view's own choice is what turns it enhanced.
    descriptor.renderSettings.setRendererOverrides({ volumetric: { mode: ERenderVolumetricMode.ENGINE } });

    const { getByRole, getByText, view } = renderLevelView(
      <LevelEnhancedFeatureSection feature={"volumetric"} />,
      descriptor
    );

    expect(getByText(explainRenderVolumetricMode(ERenderVolumetricMode.ENGINE))).toBeInTheDocument();
    expect(getByRole("slider", { name: "Intensity" })).toBeDisabled();

    await userEvent.click(getByRole("checkbox", { name: "Enhanced volumetric light" }));

    expect(view.features.volumetric).toEqual({ mode: ERenderVolumetricMode.ENHANCED });
    expect(getByRole("slider", { name: "Intensity" })).toBeEnabled();
    expect(getByRole("slider", { name: "Intensity" })).toHaveAttribute("aria-valuetext", "1.00");
    expect(getByRole("slider", { name: "Anisotropy" })).toHaveAttribute("aria-valuetext", "0.25");
    // The Base preset's own reach, which the settings keep.
    expect(getByRole("slider", { name: "Distance" })).toHaveAttribute("aria-valuetext", "80 m");

    await userEvent.click(getByRole("button", { name: "Reset the volumetric light to the preset" }));

    expect(view.features.volumetric).toEqual({});
    expect(getByRole("slider", { name: "Intensity" })).toBeDisabled();
  });

  it("sets one value of the enhanced feature for the view, the rest left to the settings", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ debanding: DEFAULT_RENDER_DEBANDING_SETTINGS });

    const { getByRole, view } = renderLevelView(<LevelEnhancedFeatureSection feature={"debanding"} />, descriptor);

    await userEvent.click(getByRole("checkbox", { name: "Debanding" }));

    expect(getByRole("slider", { name: "Debanding radius" })).toHaveAttribute("aria-valuetext", "48 px");
    expect(getByRole("slider", { name: "Debanding passes" })).toHaveAttribute("aria-valuetext", "2 times");

    getByRole("slider", { name: "Debanding passes" }).focus();
    await userEvent.keyboard("{ArrowRight}");

    expect(view.features.debanding).toEqual({ mode: ERenderDebandingMode.ENHANCED, passes: 3 });
  });

  it("names a feature by its own toggle and values", async () => {
    const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

    descriptor.renderSettings.setRendererOverrides({ glass: { mode: ERenderGlassMode.ENGINE } });

    const { getByRole, view } = renderLevelView(<LevelEnhancedFeatureSection feature={"glass"} />, descriptor);

    await userEvent.click(getByRole("checkbox", { name: "Enhanced glass" }));

    expect(view.features.glass).toEqual({ mode: ERenderGlassMode.ENHANCED });
    expect(getByRole("slider", { name: "Refraction" })).toHaveAttribute("aria-valuetext", "1.00");
  });
});
