import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderLightShadowFilter } from "@/core/ipc/types/xrf-renderer";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features/render-shadow-choices";
import { TRenderLightsSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The local lights: whether they light the level, and whether the level file's own join the spawned ones. */
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

        <CheckboxFormRow
          label={"Lamp shafts"}
          description={"Shafts of light from the shadowed lamps. Game setting: r2_volumetric_lights."}
          isChecked={lights.isVolumetric}
          onChange={(isVolumetric: boolean) => onSet({ isVolumetric })}
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
