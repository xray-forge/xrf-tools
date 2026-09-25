import { useInjection } from "@wirestate/react";
import { ERendererLightShadowFilter, IRendererLightsSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import { RENDER_LIGHT_SHADOW_FILTER_OPTIONS } from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

/** The local lights: whether they light the level, and whether the level file's own join the spawned ones. */
export function SettingsRendererLights(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const lights: IRendererLightsSettings = settingsService.rendererFeatures.lights;

  function set(part: Partial<IRendererLightsSettings>): void {
    settingsService.setRendererOverrides({ lights: part });
  }

  return (
    <DetailSection
      title={"Lights"}
      description={
        "The lamps the game spawns on a level, point lights and spots through their projectors, animated as the game " +
        "animates them."
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Lights"}
          description={"Off, only the sun and the baked light reach the level."}
          isChecked={lights.isEnabled}
          onChange={(isEnabled: boolean) => set({ isEnabled })}
        />

        <CheckboxFormRow
          label={"Shadows"}
          description={
            "The lamps the game shadows cast their shadows, each drawn once into an atlas and kept while nothing it " +
            "casts from changes. A level fills it over its first frames."
          }
          isChecked={lights.isShadowed}
          onChange={(isShadowed: boolean) => set({ isShadowed })}
        />

        <ChoiceFormRow
          label={"Shadow filter"}
          description={
            "How a shadow's edge is softened: the engine's four comparisons a texel apart, or Anomaly's penumbra, " +
            "wider the further the caster stands from what it shades, at a larger bias."
          }
          options={RENDER_LIGHT_SHADOW_FILTER_OPTIONS}
          value={lights.shadowFilter}
          onChange={(shadowFilter: ERendererLightShadowFilter) => set({ shadowFilter })}
        />

        <CheckboxFormRow
          label={"Level lights"}
          description={
            "The level file's own lights, which the game draws only with r2_allow_r1_lights. Its light maps already " +
            "hold them, so they light twice."
          }
          isChecked={lights.isLevelLights}
          onChange={(isLevelLights: boolean) => set({ isLevelLights })}
        />
      </div>
    </DetailSection>
  );
}
