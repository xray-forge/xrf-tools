import { useInjection } from "@wirestate/react";
import { ERendererAmbientOcclusionQuality, IRendererAmbientOcclusionSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import {
  formatOcclusionRadius,
  formatOcclusionStrength,
  RENDER_AMBIENT_OCCLUSION_LIMITS,
  RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS,
} from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The screen's ambient occlusion: whether it is drawn, how far it reaches, how dark and how fine. */
export function SettingsRendererAmbientOcclusion(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const occlusion: IRendererAmbientOcclusionSettings = settingsService.rendererFeatures.ambientOcclusion;

  const onSet = useRendererOverride("ambientOcclusion");

  return (
    <DetailSection title={"Ambient occlusion"} description={"Screen-space GTAO, the counterpart of the game's SSAO."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Ambient occlusion"}
          description={"Off, only baked occlusion remains."}
          isChecked={occlusion.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <ChoiceFormRow
          label={"Quality"}
          description={"Samples per pixel. Higher is smoother and more expensive."}
          options={RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS}
          value={occlusion.quality}
          onChange={(quality: ERendererAmbientOcclusionQuality) => onSet({ quality })}
        />

        <SliderFormRow
          label={"Radius"}
          description={"Occlusion radius in metres."}
          value={occlusion.radius}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.radius}
          format={formatOcclusionRadius}
          onChange={(radius: number) => onSet({ radius })}
        />

        <SliderFormRow
          label={"Strength"}
          description={"Occlusion intensity. 1 matches XeGTAO."}
          value={occlusion.strength}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.strength}
          format={formatOcclusionStrength}
          onChange={(strength: number) => onSet({ strength })}
        />
      </div>
    </DetailSection>
  );
}
