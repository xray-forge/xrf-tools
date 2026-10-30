import { useInjection } from "@wirestate/react";
import { RenderLightFalloff, RenderLightShadowFilter, RenderLightShafts } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  explainRenderLightFalloff,
  RENDER_LIGHT_FALLOFF_OPTIONS,
} from "@/core/render/lib/features/render-light-falloff-choices";
import {
  explainRenderLightShafts,
  formatShaftStrength,
  RENDER_LIGHT_SHAFTS_OPTIONS,
  RENDER_SHAFT_STRENGTH_LIMITS,
} from "@/core/render/lib/features/render-light-shafts-choices";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features/render-shadow-choices";
import { TRenderLightsSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/**
 * The local lights: whether they light the level, whether the level file's own join the spawned ones, and which draw
 * shafts how strongly.
 */
export function SettingsRendererLights(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const lights: TRenderLightsSettings = renderSettingsService.rendererFeatures.lights;

  const onSet = useRendererOverride("lights");

  return (
    <DetailSection title={"Lamps"} description={"The level's point and spot lamps."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Lamps"}
          description={"Off, only the sun and baked lighting remain."}
          isChecked={lights.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <CheckboxFormRow
          label={"Shadows"}
          description={"Shadows from lights the game marks as casting. Cached in an atlas and redrawn only on change."}
          isChecked={lights.isShadowed}
          onChange={(isShadowed: boolean) => onSet({ isShadowed })}
        />

        <ChoiceFormRow
          label={"Shadow filter"}
          description={
            "Engine: the game's fixed four-tap filter. Soft: Anomaly's PCSS, wider with distance from the caster."
          }
          options={RENDER_LIGHT_SHADOW_FILTER_OPTIONS}
          value={lights.shadowFilter}
          onChange={(shadowFilter: RenderLightShadowFilter) => onSet({ shadowFilter })}
        />

        <ChoiceFormRow
          label={"Falloff"}
          description={explainRenderLightFalloff(lights.falloff)}
          options={RENDER_LIGHT_FALLOFF_OPTIONS}
          value={lights.falloff}
          onChange={(falloff: RenderLightFalloff) => onSet({ falloff })}
        />

        <CheckboxFormRow
          label={"Lamp shafts"}
          description={"Shafts of light from the shadowed lamps. Game setting: r2_volumetric_lights."}
          isChecked={lights.isVolumetric}
          onChange={(isVolumetric: boolean) => onSet({ isVolumetric })}
        />

        <ChoiceFormRow
          label={"Shafts from"}
          description={explainRenderLightShafts(lights.shafts)}
          options={RENDER_LIGHT_SHAFTS_OPTIONS}
          value={lights.shafts}
          isDisabled={!lights.isVolumetric}
          onChange={(shafts: RenderLightShafts) => onSet({ shafts })}
        />

        <SliderFormRow
          label={"Shaft strength"}
          description={"How strong every lamp's shafts are against the game's, in either volumetric light."}
          value={lights.shaftStrength}
          {...RENDER_SHAFT_STRENGTH_LIMITS}
          isDisabled={!lights.isVolumetric}
          format={formatShaftStrength}
          onChange={(shaftStrength: number) => onSet({ shaftStrength })}
        />

        <CheckboxFormRow
          label={"Level lights"}
          description={
            "The level file's static lights, already in its lightmaps, so they add on top. Game setting: r2_allow_r1_lights."
          }
          isChecked={lights.isLevelLights}
          onChange={(isLevelLights: boolean) => onSet({ isLevelLights })}
        />
      </div>
    </DetailSection>
  );
}
