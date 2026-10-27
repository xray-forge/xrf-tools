import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ERenderIndirectLightMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import { formatOcclusionRadius } from "@/core/render/lib/features/render-ambient-occlusion-choices";
import {
  explainRenderIndirectLightMode,
  formatIndirectLightIntensity,
  RENDER_INDIRECT_LIGHT_LIMITS,
} from "@/core/render/lib/features/render-indirect-light-choices";
import { TRenderIndirectLightSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The light the frame's surfaces bounce onto each other: whether it is gathered, how much and from how far. */
export function SettingsRendererIndirectLight(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const indirectLight: TRenderIndirectLightSettings = renderSettingsService.rendererFeatures.indirectLight;
  const isEnhanced: boolean = indirectLight.mode === ERenderIndirectLightMode.ENHANCED;

  const onSet = useRendererOverride("indirectLight");

  return (
    <DetailSection title={"Indirect light"} description={"Light the surfaces bounce onto each other."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Indirect light"}
          description={explainRenderIndirectLightMode(indirectLight.mode)}
          isChecked={isEnhanced}
          onChange={(isOn: boolean) =>
            onSet({ mode: isOn ? ERenderIndirectLightMode.ENHANCED : ERenderIndirectLightMode.ENGINE })
          }
        />

        <SliderFormRow
          label={"Intensity"}
          description={"How much of the bounced light is added: 1 is all of it."}
          value={indirectLight.intensity}
          {...RENDER_INDIRECT_LIGHT_LIMITS.intensity}
          isDisabled={!isEnhanced}
          format={formatIndirectLightIntensity}
          onChange={(intensity: number) => onSet({ intensity })}
        />

        <SliderFormRow
          label={"Bounce radius"}
          description={"Metres around a point the bounced light is gathered from."}
          value={indirectLight.radius}
          {...RENDER_INDIRECT_LIGHT_LIMITS.radius}
          isDisabled={!isEnhanced}
          format={formatOcclusionRadius}
          onChange={(radius: number) => onSet({ radius })}
        />

        <SliderFormRow
          label={"Bounce steps"}
          description={"Steps the search takes past the occlusion's radius out to the bounce radius."}
          value={indirectLight.steps}
          {...RENDER_INDIRECT_LIGHT_LIMITS.steps}
          isDisabled={!isEnhanced}
          format={formatCount}
          onChange={(steps: number) => onSet({ steps })}
        />
      </div>
    </DetailSection>
  );
}
