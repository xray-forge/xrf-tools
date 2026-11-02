import { useInjection } from "@wirestate/react";
import { ERenderRainMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { explainRenderRainMode, RENDER_RAIN_SETTINGS } from "@/core/render/lib/features/world/render-rain-choices";
import { IRenderFeatureSettings, TRenderRainSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The rain's puddles: whether they gather, how many, how far they reflect and ripple, and whether they fill relief. */
export function SettingsRendererPuddles(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const rain: TRenderRainSettings = features.rain;

  const onSet = useRendererOverride("rain");

  return (
    <DetailSection title={"Puddles"} description={"Water gathering on flat ground while it rains."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={RENDER_RAIN_SETTINGS.mode.label}
          description={explainRenderRainMode(rain.mode)}
          isChecked={rain.mode === ERenderRainMode.ENHANCED}
          onChange={(isOn: boolean) => onSet({ mode: isOn ? ERenderRainMode.ENHANCED : ERenderRainMode.ENGINE })}
        />

        <RenderSettingRows
          settings={RENDER_RAIN_SETTINGS}
          names={["puddles", "reflectivity", "distance", "ripples", "pace", "isReliefFlooded"]}
          group={rain}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
