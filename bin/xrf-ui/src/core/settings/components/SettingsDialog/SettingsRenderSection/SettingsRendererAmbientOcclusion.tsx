import { useInjection } from "@wirestate/react";
import { ERendererAmbientOcclusionQuality, IRendererAmbientOcclusionSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import { describeRenderAmbientOcclusionQuality, RENDER_AMBIENT_OCCLUSION_LIMITS } from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";
import { formatNumber } from "@/lib/format/number";

const QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERendererAmbientOcclusionQuality>> = Object.values(
  ERendererAmbientOcclusionQuality
).map((value: ERendererAmbientOcclusionQuality) => ({ label: describeRenderAmbientOcclusionQuality(value), value }));

/** The screen's ambient occlusion: whether it is drawn, how far it reaches, how dark and how fine. */
export function SettingsRendererAmbientOcclusion(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const occlusion: IRendererAmbientOcclusionSettings = settingsService.rendererFeatures.ambientOcclusion;

  function set(part: Partial<IRendererAmbientOcclusionSettings>): void {
    settingsService.setRendererOverrides({ ambientOcclusion: part });
  }

  return (
    <DetailSection
      title={"Ambient occlusion"}
      description={
        "GTAO from the frame's depth, as the game's SSAO: it darkens the sky's and the ambient light in creases, " +
        "corners and under what stands close, over the occlusion the level baked."
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Ambient occlusion"}
          description={"Off, only the level's baked occlusion shades what the sky cannot reach."}
          isChecked={occlusion.isEnabled}
          onChange={(isEnabled: boolean) => set({ isEnabled })}
        />

        <ChoiceFormRow
          label={"Quality"}
          description={"How many directions and steps each pixel searches. Higher is smoother and costs more."}
          options={QUALITY_OPTIONS}
          value={occlusion.quality}
          onChange={(quality: ERendererAmbientOcclusionQuality) => set({ quality })}
        />

        <SliderFormRow
          label={"Radius"}
          description={"Metres around a point that what stands there shades it from."}
          value={occlusion.radius}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.radius}
          format={(value: number) => `${formatNumber(value, 2)} m`}
          onChange={(radius: number) => set({ radius })}
        />

        <SliderFormRow
          label={"Strength"}
          description={"How dark the occlusion goes: one as XeGTAO draws it, none at zero."}
          value={occlusion.strength}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.strength}
          format={(value: number) => formatNumber(value, 1)}
          onChange={(strength: number) => set({ strength })}
        />
      </div>
    </DetailSection>
  );
}
