import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderTonemapMode } from "@/core/ipc/types/xrf-renderer";
import {
  explainRenderTonemapMode,
  formatExposure,
  formatLowLuminance,
  RENDER_EXPOSURE_LIMITS,
  RENDER_TONEMAP_MODE_OPTIONS,
} from "@/core/render/lib/features/render-exposure-choices";
import { TRenderExposureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The frame's exposure: whether it adapts to the frame's own luminance, towards what, and how fast. */
export function SettingsRendererExposure(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const exposure: TRenderExposureSettings = renderSettingsService.rendererFeatures.exposure;

  const onSet = useRendererOverride("exposure");

  return (
    <DetailSection title={"Exposure"} description={"Automatic exposure, as the engine's tonemapper applies it."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Tonemap"}
          description={explainRenderTonemapMode(renderSettingsService.rendererFeatures.tonemap)}
          options={RENDER_TONEMAP_MODE_OPTIONS}
          value={renderSettingsService.rendererFeatures.tonemap}
          onChange={(tonemap: RenderTonemapMode) => renderSettingsService.setRendererOverrides({ tonemap })}
        />

        <CheckboxFormRow
          label={"Auto exposure"}
          description={
            "Adapts brightness to the frame's average luminance. Off, the engine's fixed daylight exposure is used."
          }
          isChecked={exposure.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <SliderFormRow
          label={"Middle gray"}
          description={"Target luminance. Game setting: r2_tonemap_middlegray."}
          value={exposure.middleGray}
          {...RENDER_EXPOSURE_LIMITS.middleGray}
          isDisabled={!exposure.isEnabled}
          format={formatExposure}
          onChange={(middleGray: number) => onSet({ middleGray })}
        />

        <SliderFormRow
          label={"Strength"}
          description={"Share of the adaptation applied. Game setting: r2_tonemap_amount."}
          value={exposure.amount}
          {...RENDER_EXPOSURE_LIMITS.amount}
          isDisabled={!exposure.isEnabled}
          format={formatExposure}
          onChange={(amount: number) => onSet({ amount })}
        />

        <SliderFormRow
          label={"Luminance floor"}
          description={"Lowest luminance the meter reads. Game setting: r2_tonemap_lowlum."}
          value={exposure.lowLuminance}
          {...RENDER_EXPOSURE_LIMITS.lowLuminance}
          isDisabled={!exposure.isEnabled}
          format={formatLowLuminance}
          onChange={(lowLuminance: number) => onSet({ lowLuminance })}
        />

        <SliderFormRow
          label={"Adaptation speed"}
          description={"How quickly the exposure follows the frame. Game setting: r2_tonemap_adaptation."}
          value={exposure.adaptation}
          {...RENDER_EXPOSURE_LIMITS.adaptation}
          isDisabled={!exposure.isEnabled}
          format={formatExposure}
          onChange={(adaptation: number) => onSet({ adaptation })}
        />
      </div>
    </DetailSection>
  );
}
