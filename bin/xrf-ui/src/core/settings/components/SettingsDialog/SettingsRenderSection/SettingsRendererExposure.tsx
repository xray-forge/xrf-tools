import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { IRendererExposureSettings } from "@/core/render/lib/contract/renderer-exposure-settings";
import { formatExposure, formatLowLuminance, RENDER_EXPOSURE_LIMITS } from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The frame's exposure: whether it adapts to the frame's own luminance, towards what, and how fast. */
export function SettingsRendererExposure(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const exposure: IRendererExposureSettings = settingsService.rendererFeatures.exposure;

  const onSet = useRendererOverride("exposure");

  return (
    <DetailSection title={"Exposure"} description={"Automatic exposure, as the engine's tonemapper applies it."}>
      <div className={"mt-4 flex flex-col gap-6"}>
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
          description={"Target luminance (r2_tonemap_middlegray)."}
          value={exposure.middleGray}
          {...RENDER_EXPOSURE_LIMITS.middleGray}
          isDisabled={!exposure.isEnabled}
          format={formatExposure}
          onChange={(middleGray: number) => onSet({ middleGray })}
        />

        <SliderFormRow
          label={"Strength"}
          description={"Share of the adaptation applied (r2_tonemap_amount)."}
          value={exposure.amount}
          {...RENDER_EXPOSURE_LIMITS.amount}
          isDisabled={!exposure.isEnabled}
          format={formatExposure}
          onChange={(amount: number) => onSet({ amount })}
        />

        <SliderFormRow
          label={"Luminance floor"}
          description={"Lowest luminance the meter reads (r2_tonemap_lowlum)."}
          value={exposure.lowLuminance}
          {...RENDER_EXPOSURE_LIMITS.lowLuminance}
          isDisabled={!exposure.isEnabled}
          format={formatLowLuminance}
          onChange={(lowLuminance: number) => onSet({ lowLuminance })}
        />

        <SliderFormRow
          label={"Adaptation speed"}
          description={"How quickly exposure follows the frame (r2_tonemap_adaptation)."}
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
