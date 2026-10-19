import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ERenderRainDropsMode } from "@/core/ipc/types/xrf-renderer";
import { explainRenderRainDropsMode } from "@/core/render/lib/features/render-rain-choices";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The falling rain: the game's streaks, or drops bending the scene behind them. */
export function SettingsRendererRainDrops(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const rain: TRenderRainSettings = renderSettingsService.rendererFeatures.rain;

  const onSet = useRendererOverride("rain");

  return (
    <DetailSection title={"Rain"} description={"How the rain falls."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Refracting drops"}
          description={explainRenderRainDropsMode(rain.drops)}
          isChecked={rain.drops === ERenderRainDropsMode.ENHANCED}
          onChange={(isOn: boolean) =>
            onSet({ drops: isOn ? ERenderRainDropsMode.ENHANCED : ERenderRainDropsMode.ENGINE })
          }
        />
      </div>
    </DetailSection>
  );
}
