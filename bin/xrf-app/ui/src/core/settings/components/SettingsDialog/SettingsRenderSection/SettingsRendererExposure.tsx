import { useInjection } from "@wirestate/react";
import {
  ERenderLightLevel,
  ERenderLightModel,
  RenderDisplayTransform,
  RenderLightLevel,
  RenderLightModel,
  RenderTonemapMode,
} from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  explainRenderDisplayTransform,
  RENDER_DISPLAY_TRANSFORM_OPTIONS,
} from "@/core/render/lib/features/render-display-transform-choices";
import {
  explainRenderTonemapMode,
  formatExposure,
  formatLowLuminance,
  formatStops,
  RENDER_EXPOSURE_LIMITS,
  RENDER_TONEMAP_MODE_OPTIONS,
} from "@/core/render/lib/features/render-exposure-choices";
import { formatGrading, RENDER_GRADING_LIMITS } from "@/core/render/lib/features/render-grading-choices";
import {
  explainRenderLightLevel,
  formatSunTint,
  getRenderLightLevelOptions,
  RENDER_SUN_TINT_LIMITS,
} from "@/core/render/lib/features/render-light-level-choices";
import {
  explainRenderLightModel,
  RENDER_LIGHT_MODEL_OPTIONS,
} from "@/core/render/lib/features/render-light-model-choices";
import { TRenderExposureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The frame's exposure: whether it adapts to the frame's own luminance, towards what, and how fast. */
export function SettingsRendererExposure(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const exposure: TRenderExposureSettings = renderSettingsService.rendererFeatures.exposure;

  const onSet = useRendererOverride("exposure");
  // The display transform and the exposure compensation are the physical light's alone; the tonemap mode and the
  // engine meter's values the engine light's, as the physical light's scene is always linear and metered by histogram.
  const isPhysical: boolean = renderSettingsService.rendererFeatures.lightModel === ERenderLightModel.PHYSICAL;
  const isMetered: boolean = exposure.isEnabled && !isPhysical;

  return (
    <DetailSection
      title={"Exposure"}
      description={"Automatic exposure: the engine tonemapper's meter, or the physical light's histogram."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Light"}
          description={explainRenderLightModel(renderSettingsService.rendererFeatures.lightModel)}
          options={RENDER_LIGHT_MODEL_OPTIONS}
          value={renderSettingsService.rendererFeatures.lightModel}
          onChange={(lightModel: RenderLightModel) => renderSettingsService.setRendererOverrides({ lightModel })}
        />

        <ChoiceFormRow
          label={"Light levels"}
          description={explainRenderLightLevel(renderSettingsService.rendererFeatures.lightLevel)}
          options={getRenderLightLevelOptions(renderSettingsService.isDevLightLevelAvailable)}
          value={renderSettingsService.rendererFeatures.lightLevel}
          isDisabled={!isPhysical}
          onChange={(lightLevel: RenderLightLevel) => renderSettingsService.setRendererOverrides({ lightLevel })}
        />

        <SliderFormRow
          label={"Sun tint"}
          description={"How much of the weather's own sun colour the real-world light levels keep in the sun's."}
          value={renderSettingsService.rendererFeatures.sunTint}
          {...RENDER_SUN_TINT_LIMITS}
          isDisabled={!isPhysical || renderSettingsService.rendererFeatures.lightLevel !== ERenderLightLevel.PHYSICAL}
          format={formatSunTint}
          onChange={(sunTint: number) => renderSettingsService.setRendererOverrides({ sunTint })}
        />

        <ChoiceFormRow
          label={"Display transform"}
          description={explainRenderDisplayTransform(renderSettingsService.rendererFeatures.displayTransform)}
          options={RENDER_DISPLAY_TRANSFORM_OPTIONS}
          value={renderSettingsService.rendererFeatures.displayTransform}
          isDisabled={!isPhysical}
          onChange={(displayTransform: RenderDisplayTransform) =>
            renderSettingsService.setRendererOverrides({ displayTransform })
          }
        />

        <SliderFormRow
          label={"Display contrast"}
          description={"Stops of the physical light's frame about middle grey, raised before its display transform."}
          value={renderSettingsService.rendererFeatures.grading.contrast}
          {...RENDER_GRADING_LIMITS.contrast}
          isDisabled={!isPhysical}
          format={formatGrading}
          onChange={(contrast: number) =>
            renderSettingsService.setRendererOverrides({
              grading: { ...renderSettingsService.rendererFeatures.grading, contrast },
            })
          }
        />

        <SliderFormRow
          label={"Display saturation"}
          description={"How far the physical light's frame is taken from grey before its display transform."}
          value={renderSettingsService.rendererFeatures.grading.saturation}
          {...RENDER_GRADING_LIMITS.saturation}
          isDisabled={!isPhysical}
          format={formatGrading}
          onChange={(saturation: number) =>
            renderSettingsService.setRendererOverrides({
              grading: { ...renderSettingsService.rendererFeatures.grading, saturation },
            })
          }
        />

        <ChoiceFormRow
          label={"Tonemap"}
          description={explainRenderTonemapMode(renderSettingsService.rendererFeatures.tonemap)}
          options={RENDER_TONEMAP_MODE_OPTIONS}
          value={renderSettingsService.rendererFeatures.tonemap}
          isDisabled={isPhysical}
          onChange={(tonemap: RenderTonemapMode) => renderSettingsService.setRendererOverrides({ tonemap })}
        />

        <CheckboxFormRow
          label={"Auto exposure"}
          description={
            "Adapts brightness to the frame's average luminance. Off, the engine's fixed daylight exposure is used, or the compensation under the physical light."
          }
          isChecked={exposure.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <SliderFormRow
          label={"Compensation"}
          description={
            "Stops the physical light's exposure lifts the frame over the level it meters it at, or lowers it."
          }
          value={exposure.compensation}
          {...RENDER_EXPOSURE_LIMITS.compensation}
          isDisabled={!isPhysical}
          format={formatStops}
          onChange={(compensation: number) => onSet({ compensation })}
        />

        <SliderFormRow
          label={"Middle gray"}
          description={"Target luminance. Game setting: r2_tonemap_middlegray."}
          value={exposure.middleGray}
          {...RENDER_EXPOSURE_LIMITS.middleGray}
          isDisabled={!isMetered}
          format={formatExposure}
          onChange={(middleGray: number) => onSet({ middleGray })}
        />

        <SliderFormRow
          label={"Strength"}
          description={"Share of the adaptation applied. Game setting: r2_tonemap_amount."}
          value={exposure.amount}
          {...RENDER_EXPOSURE_LIMITS.amount}
          isDisabled={!isMetered}
          format={formatExposure}
          onChange={(amount: number) => onSet({ amount })}
        />

        <SliderFormRow
          label={"Luminance floor"}
          description={"Lowest luminance the meter reads. Game setting: r2_tonemap_lowlum."}
          value={exposure.lowLuminance}
          {...RENDER_EXPOSURE_LIMITS.lowLuminance}
          isDisabled={!isMetered}
          format={formatLowLuminance}
          onChange={(lowLuminance: number) => onSet({ lowLuminance })}
        />

        <SliderFormRow
          label={"Adaptation speed"}
          description={"How quickly the exposure follows the frame. Game setting: r2_tonemap_adaptation."}
          value={exposure.adaptation}
          {...RENDER_EXPOSURE_LIMITS.adaptation}
          isDisabled={!isMetered}
          format={formatExposure}
          onChange={(adaptation: number) => onSet({ adaptation })}
        />
      </div>
    </DetailSection>
  );
}
