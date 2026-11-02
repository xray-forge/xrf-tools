import { useInjection } from "@wirestate/react";
import { ERenderRainDropsMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { explainRenderRainDropsMode, RENDER_RAIN_SETTINGS } from "@/core/render/lib/features/world/render-rain-choices";
import { IRenderFeatureSettings, TRenderRainSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The falling rain: the game's streaks, or drops bending the scene behind them, and their rings on the water. */
export function SettingsRendererRainDrops(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const rain: TRenderRainSettings = features.rain;

  const onSet = useRendererOverride("rain");

  return (
    <DetailSection title={"Rain"} description={"How the rain falls."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={RENDER_RAIN_SETTINGS.drops.label}
          description={explainRenderRainDropsMode(rain.drops)}
          isChecked={rain.drops === ERenderRainDropsMode.ENHANCED}
          onChange={(isOn: boolean) =>
            onSet({ drops: isOn ? ERenderRainDropsMode.ENHANCED : ERenderRainDropsMode.ENGINE })
          }
        />

        <RenderSettingRows
          settings={RENDER_RAIN_SETTINGS}
          names={["dropSquare", "ringDistance", "ringStrength", "ringSize"]}
          group={rain}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
