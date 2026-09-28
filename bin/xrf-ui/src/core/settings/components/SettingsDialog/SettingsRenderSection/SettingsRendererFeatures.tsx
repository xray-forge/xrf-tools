import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import {
  ERendererAntialiasing,
  ERendererPreset,
  ERendererRenderScale,
  IRendererFeatureSettings,
  isRendererFeatureChoiceCustom,
} from "@xrf/renderer";
import { ReactElement } from "react";

import {
  formatExposure,
  formatLowLuminance,
  formatSharpening,
  RENDER_ANTIALIASING_OPTIONS,
  RENDER_EXPOSURE_LIMITS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
} from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

const PRESET_LABELS: Record<ERendererPreset, string> = {
  [ERendererPreset.BASE]: "Base",
  [ERendererPreset.EDITING]: "Editing",
};

const PRESET_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERendererPreset>> = Object.values(ERendererPreset).map(
  (value: ERendererPreset) => ({ label: PRESET_LABELS[value], value })
);

/** Which preset the renderer's features follow, and the features that are not a level's alone. */
export function SettingsRendererFeatures(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const { preset } = settingsService.rendererChoice;
  const features: IRendererFeatureSettings = settingsService.rendererFeatures;
  const isCustom: boolean = isRendererFeatureChoiceCustom(settingsService.rendererChoice);

  return (
    <DetailSection
      title={"Features"}
      description={"What every viewport draws with: a preset, and whatever is changed on top of it."}
      fact={isCustom ? `Custom, from ${PRESET_LABELS[preset]}` : PRESET_LABELS[preset]}
      action={
        isCustom ? (
          <Button size={"small"} onClick={() => settingsService.setRendererPreset(preset)}>
            Back to {PRESET_LABELS[preset]}
          </Button>
        ) : null
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Preset"}
          description={"Base is the game's own defaults. Editing puts responsiveness before looks."}
          options={PRESET_OPTIONS}
          value={preset}
          onChange={settingsService.setRendererPreset}
        />

        <ChoiceFormRow
          label={"Antialiasing"}
          description={
            "How the frame's edges are smoothed. SMAA is crisp and stable; FXAA is cheaper and softer. TAA blends " +
            "each frame with the ones before, which also smooths foliage and thin wires, and settles when the view " +
            "stops moving. FSR 2 does the same as AMD's upscaler, holding thin features and the blended surfaces " +
            "steadier."
          }
          options={RENDER_ANTIALIASING_OPTIONS}
          value={features.antialiasing}
          onChange={(antialiasing: ERendererAntialiasing) => settingsService.setRendererOverrides({ antialiasing })}
        />

        <ChoiceFormRow
          label={"Render scale"}
          description={
            "How much of each side the scene is drawn at before it is upscaled to the view: by TAA or FSR 2 from the " +
            "frames before as much as this one, and by FSR 1 from this frame alone for the other modes. Less costs " +
            "less for every pass that is paid per pixel."
          }
          options={RENDER_SCALE_OPTIONS}
          value={features.upscaling.scale}
          onChange={(scale: ERendererRenderScale) => settingsService.setRendererOverrides({ upscaling: { scale } })}
        />

        <SliderFormRow
          label={"Sharpening"}
          description={"How much the upscaled frame is sharpened after, as FSR's RCAS does. None at native."}
          value={features.upscaling.sharpening}
          {...RENDER_SHARPENING_LIMITS}
          isDisabled={features.upscaling.scale === ERendererRenderScale.NATIVE}
          format={formatSharpening}
          onChange={(sharpening: number) => settingsService.setRendererOverrides({ upscaling: { sharpening } })}
        />

        <CheckboxFormRow
          label={"Exposure adaptation"}
          description={
            "Brightens or darkens the level towards the middle gray by its average luminance, as the engine's " +
            "r2_tonemap does. Off, the frame is drawn at the engine's noon scale."
          }
          isChecked={features.exposure.isEnabled}
          onChange={(isEnabled: boolean) => settingsService.setRendererOverrides({ exposure: { isEnabled } })}
        />

        <SliderFormRow
          label={"Middle gray"}
          description={"The luminance the frame is brought towards: r2_tonemap_middlegray."}
          value={features.exposure.middleGray}
          {...RENDER_EXPOSURE_LIMITS.middleGray}
          isDisabled={!features.exposure.isEnabled}
          format={formatExposure}
          onChange={(middleGray: number) => settingsService.setRendererOverrides({ exposure: { middleGray } })}
        />

        <SliderFormRow
          label={"Adaptation amount"}
          description={"How much of the adaptation applies: r2_tonemap_amount."}
          value={features.exposure.amount}
          {...RENDER_EXPOSURE_LIMITS.amount}
          isDisabled={!features.exposure.isEnabled}
          format={formatExposure}
          onChange={(amount: number) => settingsService.setRendererOverrides({ exposure: { amount } })}
        />

        <SliderFormRow
          label={"Low luminance"}
          description={"What the measured luminance is floored at: r2_tonemap_lowlum."}
          value={features.exposure.lowLuminance}
          {...RENDER_EXPOSURE_LIMITS.lowLuminance}
          isDisabled={!features.exposure.isEnabled}
          format={formatLowLuminance}
          onChange={(lowLuminance: number) => settingsService.setRendererOverrides({ exposure: { lowLuminance } })}
        />

        <SliderFormRow
          label={"Adaptation speed"}
          description={"How fast the exposure follows the frame: r2_tonemap_adaptation."}
          value={features.exposure.adaptation}
          {...RENDER_EXPOSURE_LIMITS.adaptation}
          isDisabled={!features.exposure.isEnabled}
          format={formatExposure}
          onChange={(adaptation: number) => settingsService.setRendererOverrides({ exposure: { adaptation } })}
        />

        <CheckboxFormRow
          label={"Water"}
          description={"Draws the water as the engine does: rippled and reflecting the sky."}
          isChecked={features.water.isEnabled}
          onChange={(isEnabled: boolean) => settingsService.setRendererOverrides({ water: { isEnabled } })}
        />

        <CheckboxFormRow
          label={"Soft water"}
          description={"Fades the water by how deep it is, and lays foam in the shallows: r2_soft_water."}
          isChecked={features.water.isSoft}
          isDisabled={!features.water.isEnabled}
          onChange={(isSoft: boolean) => settingsService.setRendererOverrides({ water: { isSoft } })}
        />

        <CheckboxFormRow
          label={"Water distortion"}
          description={"Moves what is seen through the water, as the engine's distortion does."}
          isChecked={features.water.isDistorted}
          isDisabled={!features.water.isEnabled}
          onChange={(isDistorted: boolean) => settingsService.setRendererOverrides({ water: { isDistorted } })}
        />

        <CheckboxFormRow
          label={"Occlusion culling"}
          description={"Culls the level's static draws that nearer ones hide, twice a frame against its depth."}
          isChecked={features.isOcclusionCulled}
          onChange={(isOcclusionCulled: boolean) => settingsService.setRendererOverrides({ isOcclusionCulled })}
        />

        <CheckboxFormRow
          label={"GPU timings"}
          description={"Times every pass on the GPU for the readout."}
          isChecked={features.isGpuTimed}
          onChange={(isGpuTimed: boolean) => settingsService.setRendererOverrides({ isGpuTimed })}
        />
      </div>
    </DetailSection>
  );
}
