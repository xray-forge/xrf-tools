import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { ERenderLightShafts } from "@xrf/ipc/types/xrf-renderer";

import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  explainRenderLightShafts,
  RENDER_LIGHTS_SETTINGS,
} from "@/core/render/lib/features/lighting/render-lights-choices";
import {
  IRenderFeatureSettings,
  TRenderLightsSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/choice/render-preset";
import { renderWithProviders } from "@/fixtures/utils/render";

const FEATURES: IRenderFeatureSettings = RENDER_PRESETS[ERenderPreset.BASE];

/** Lamps without shafts, which the shafts' values tune nothing of. */
const LIGHTS: TRenderLightsSettings = {
  ...FEATURES.lights,
  isVolumetric: false,
  shafts: ERenderLightShafts.ENGINE,
  shaftStrength: 1,
};

function renderEntries(onSet: (part: Partial<TRenderLightsSettings>) => void = jest.fn()): RenderResult {
  return renderWithProviders(
    <RenderSettingEntries
      settings={RENDER_LIGHTS_SETTINGS}
      names={["isVolumetric", "shafts", "shaftStrength", "isShadowed"]}
      group={LIGHTS}
      features={FEATURES}
      onSet={onSet}
    />
  );
}

describe("RenderSettingEntries", () => {
  it("draws each setting as its declaration says, disabled where its rule finds it changing nothing", () => {
    const { getByRole } = renderEntries();

    expect(getByRole("checkbox", { name: "Shafts" })).toBeEnabled();
    expect(getByRole("slider", { name: "Strength" })).toBeDisabled();
    expect(getByRole("button", { name: "Every lamp" })).toBeDisabled();
  });

  it("describes every setting by what it does, saying it under its row only where its declaration asks", () => {
    const { getAllByText, getByText } = renderEntries();

    function isShown(it: HTMLElement): boolean {
      return !it.classList.contains("sr-only");
    }

    expect(getByText(explainRenderLightShafts(ERenderLightShafts.ENGINE))).toBeInTheDocument();
    expect(getAllByText(RENDER_LIGHTS_SETTINGS.isVolumetric.description as string).some(isShown)).toBe(false);
    expect(getAllByText(RENDER_LIGHTS_SETTINGS.isShadowed.description as string).some(isShown)).toBe(false);
  });

  it("sets the one value it changes, the rest of the group left as it was", () => {
    const onSet = jest.fn<(part: Partial<TRenderLightsSettings>) => void>();
    const { getByRole } = renderEntries(onSet);

    fireEvent.click(getByRole("checkbox", { name: "Shafts" }));

    expect(onSet).toHaveBeenCalledWith({ isVolumetric: true });
  });
});

describe("RenderSettingRows", () => {
  it("says what every setting does, a choice what the value chosen does", () => {
    const { getByText } = renderWithProviders(
      <RenderSettingRows
        settings={RENDER_LIGHTS_SETTINGS}
        names={["isShadowed", "shafts"]}
        group={LIGHTS}
        features={FEATURES}
        onSet={jest.fn()}
      />
    );

    expect(getByText(RENDER_LIGHTS_SETTINGS.isShadowed.description as string)).toBeInTheDocument();
    expect(getByText(explainRenderLightShafts(ERenderLightShafts.ENGINE))).toBeInTheDocument();
  });
});
