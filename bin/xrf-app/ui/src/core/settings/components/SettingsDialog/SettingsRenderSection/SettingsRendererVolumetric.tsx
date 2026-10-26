import { useInjection } from "@wirestate/react";
import { ERenderVolumetricMode, RenderVolumetricMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import {
  explainRenderVolumetricMode,
  formatVolumetricDistance,
  formatVolumetricStrength,
  RENDER_VOLUMETRIC_LIMITS,
  RENDER_VOLUMETRIC_MODE_OPTIONS,
} from "@/core/render/lib/features/render-volumetric-choices";
import { TRenderVolumetricSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The light the air scatters: the game's shafts, or the enhanced volume, how strong, how forward and how far. */
export function SettingsRendererVolumetric(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const volumetric: TRenderVolumetricSettings = renderSettingsService.rendererFeatures.volumetric;
  const isEngine: boolean = volumetric.mode === ERenderVolumetricMode.ENGINE;

  const onSet = useRendererOverride("volumetric");

  return (
    <DetailSection title={"Volumetric light"} description={"Light scattered by the air, from the sun and the lamps."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Mode"}
          description={explainRenderVolumetricMode(volumetric.mode)}
          options={RENDER_VOLUMETRIC_MODE_OPTIONS}
          value={volumetric.mode}
          onChange={(mode: RenderVolumetricMode) => onSet({ mode })}
        />

        <SliderFormRow
          label={"Intensity"}
          description={"How strongly the volume scatters: 1 as designed."}
          value={volumetric.intensity}
          {...RENDER_VOLUMETRIC_LIMITS.intensity}
          isDisabled={isEngine}
          format={formatVolumetricStrength}
          onChange={(intensity: number) => onSet({ intensity })}
        />

        <SliderFormRow
          label={"Anisotropy"}
          description={"How far scattered light keeps the way it was going: 0 scatters it every way alike."}
          value={volumetric.anisotropy}
          {...RENDER_VOLUMETRIC_LIMITS.anisotropy}
          isDisabled={isEngine}
          format={formatVolumetricStrength}
          onChange={(anisotropy: number) => onSet({ anisotropy })}
        />

        <SliderFormRow
          label={"Distance"}
          description={"Metres along the view the volume reaches."}
          value={volumetric.distance}
          {...RENDER_VOLUMETRIC_LIMITS.distance}
          isDisabled={isEngine}
          format={formatVolumetricDistance}
          onChange={(distance: number) => onSet({ distance })}
        />
      </div>
    </DetailSection>
  );
}
