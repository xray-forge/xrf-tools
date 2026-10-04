import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderLightShadowFilter } from "@/core/ipc/types/xrf-renderer";
import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features";
import { TRenderLightsSettings } from "@/core/render/lib/settings/render-feature-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The local lights: whether they light the level, and whether the level file's own join the spawned ones. */
export function SettingsRendererLights(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const lights: TRenderLightsSettings = settingsService.rendererFeatures.lights;

  const onSet = useRendererOverride("lights");

  return (
    <DetailSection title={"Lights"} description={"Dynamic point and spot lights spawned on the level."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Dynamic lights"}
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
          label={"Level lights"}
          description={
            "Static lights from the level file (r2_allow_r1_lights). Already in the lightmaps, so they add on top."
          }
          isChecked={lights.isLevelLights}
          onChange={(isLevelLights: boolean) => onSet({ isLevelLights })}
        />
      </div>
    </DetailSection>
  );
}
