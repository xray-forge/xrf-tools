import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@xrf/ui/form/SliderFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import {
  formatOcclusionRadius,
  formatOcclusionStrength,
  RENDER_AMBIENT_OCCLUSION_LIMITS,
  RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS,
} from "@/core/render/lib/features/render-ambient-occlusion-choices";
import { TRenderAmbientOcclusionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The screen's ambient occlusion: whether it is drawn, how far it reaches, how dark, and each method's search. */
export function SettingsRendererAmbientOcclusion(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const occlusion: TRenderAmbientOcclusionSettings = renderSettingsService.rendererFeatures.ambientOcclusion;

  const onSet = useRendererOverride("ambientOcclusion");

  return (
    <DetailSection
      title={"Ambient occlusion"}
      description={
        "The game's SSAO, as GTAO or VBAO from the screen or RTAO traced, picked in a level's Ambient light popover."
      }
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Ambient occlusion"}
          description={"Off, only baked occlusion remains."}
          isChecked={occlusion.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
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
          description={"Occlusion intensity. 1 is each method's own curve."}
          value={occlusion.strength}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.strength}
          format={formatOcclusionStrength}
          onChange={(strength: number) => onSet({ strength })}
        />

        <SliderFormRow
          label={"GTAO slices"}
          description={"Directions GTAO searches about a pixel for its horizons."}
          value={occlusion.gtao.slices}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.gtaoSlices}
          format={formatCount}
          onChange={(slices: number) => onSet({ gtao: { ...occlusion.gtao, slices } })}
        />

        <SliderFormRow
          label={"GTAO steps"}
          description={"Steps each way along each of GTAO's slices."}
          value={occlusion.gtao.steps}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.gtaoSteps}
          format={formatCount}
          onChange={(steps: number) => onSet({ gtao: { ...occlusion.gtao, steps } })}
        />

        <SliderFormRow
          label={"VBAO slices"}
          description={"Directions VBAO searches about a pixel, its visibility bitmask's."}
          value={occlusion.vbao.slices}
          {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.slices}
          format={formatCount}
          onChange={(slices: number) => onSet({ vbao: { ...occlusion.vbao, slices } })}
        />

        <SliderFormRow
          label={"VBAO steps"}
          description={"Steps along each of VBAO's slices."}
          value={occlusion.vbao.steps}
          {...RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS.steps}
          format={formatCount}
          onChange={(steps: number) => onSet({ vbao: { ...occlusion.vbao, steps } })}
        />

        <SliderFormRow
          label={"RTAO rays"}
          description={"Rays RTAO traces from each point a frame, on a GPU that traces rays."}
          value={occlusion.rtao.rays}
          {...RENDER_AMBIENT_OCCLUSION_LIMITS.rays}
          format={formatCount}
          onChange={(rays: number) => onSet({ rtao: { rays } })}
        />
      </div>
    </DetailSection>
  );
}
