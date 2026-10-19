import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ERenderDebandingMode } from "@/core/ipc/types/xrf-renderer";
import {
  explainRenderDebandingMode,
  formatDebandingPasses,
  formatDebandingRadius,
  RENDER_DEBANDING_LIMITS,
} from "@/core/render/lib/features/render-debanding-choices";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** Whether the sky's colour bands are smoothed, at what quality and over how many pixels. */
export function SettingsRendererDebanding(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const debanding: TRenderDebandingSettings = renderSettingsService.rendererFeatures.debanding;
  const isEnhanced: boolean = debanding.mode === ERenderDebandingMode.ENHANCED;

  const onSet = useRendererOverride("debanding");

  return (
    <DetailSection title={"Debanding"} description={"Smooths the colour bands in the sky's gradients."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Debanding"}
          description={explainRenderDebandingMode(debanding.mode)}
          isChecked={isEnhanced}
          onChange={(isOn: boolean) =>
            onSet({ mode: isOn ? ERenderDebandingMode.ENHANCED : ERenderDebandingMode.ENGINE })
          }
        />

        <SliderFormRow
          label={"Passes"}
          description={"Times a sky pixel is averaged with its neighbours, each time further out."}
          value={debanding.passes}
          {...RENDER_DEBANDING_LIMITS.passes}
          isDisabled={!isEnhanced}
          format={formatDebandingPasses}
          onChange={(passes: number) => onSet({ passes })}
        />

        <SliderFormRow
          label={"Radius"}
          description={"Pixels around a sky pixel its neighbours are read from at most."}
          value={debanding.radius}
          {...RENDER_DEBANDING_LIMITS.radius}
          isDisabled={!isEnhanced}
          format={formatDebandingRadius}
          onChange={(radius: number) => onSet({ radius })}
        />
      </div>
    </DetailSection>
  );
}
