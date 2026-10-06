import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ERenderBloomMode } from "@/core/ipc/types/xrf-renderer";
import { LevelLookAction } from "@/core/level/components/preview/LevelLookAction";
import { ILevelFeatureOptions } from "@/core/level/lib/features";
import { ELevelLookSource } from "@/core/level/lib/look";
import { LevelLookService } from "@/core/level/services";
import { DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderEnhancedBloomSettings } from "@/core/render/lib/settings/render-feature-settings";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

function renderAction(
  bloom: TRenderEnhancedBloomSettings = DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS,
  onChangeFeatures: (features: ILevelFeatureOptions) => void = () => {}
): { look: LevelLookService } & ReturnType<typeof renderWithProviders> {
  const container: Container = mockContainer([LevelLookService]);

  return {
    ...renderWithProviders(
      <LevelLookAction bloom={bloom} features={mockLevelFeatureOptions()} onChangeFeatures={onChangeFeatures} />,
      { container }
    ),
    look: container.get(LevelLookService),
  };
}

afterEach(() => {
  window.localStorage.clear();
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
    const onChangeFeatures = jest.fn<(features: ILevelFeatureOptions) => void>();
    const { getByRole, findByRole, queryByRole, unmount } = renderAction(undefined, onChangeFeatures);

    await userEvent.click(getByRole("button", { name: "Look" }));
    await findByRole("dialog", { name: "Look" });

    expect(getByRole("checkbox", { name: "Bloom" })).toBeInTheDocument();
    expect(queryByRole("slider", { name: "Vibrance" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Enhanced" }));

    expect(onChangeFeatures).toHaveBeenLastCalledWith({
      ...mockLevelFeatureOptions(),
      enhancedBloom: { mode: ERenderBloomMode.ENHANCED },
    });

    unmount();

    const enhanced = renderAction({ ...DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS, mode: ERenderBloomMode.ENHANCED });

    await userEvent.click(enhanced.getByRole("button", { name: "Look" }));
    await enhanced.findByRole("dialog", { name: "Look" });

    expect(enhanced.queryByRole("checkbox", { name: "Bloom" })).toBeNull();
    expect(enhanced.queryByRole("slider", { name: "Radius" })).toBeNull();
    expect(enhanced.getByRole("slider", { name: "Threshold" })).toHaveAttribute("aria-valuetext", "3.5");
    expect(enhanced.getByRole("slider", { name: "Vibrance" })).toHaveAttribute("aria-valuetext", "1.5");
  });
});
