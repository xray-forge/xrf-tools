import { useInjection } from "@wirestate/react";
import { ERenderIndirectLightMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  explainRenderIndirectLightMode,
  RENDER_INDIRECT_LIGHT_SETTINGS,
} from "@/core/render/lib/features/tracing/render-indirect-light-choices";
import {
  IRenderFeatureSettings,
  TRenderIndirectLightSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The light the frame's surfaces bounce onto each other: whether it is gathered, how much and from how far. */
export function SettingsRendererIndirectLight(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const indirectLight: TRenderIndirectLightSettings = features.indirectLight;

  const onSet = useRendererOverride("indirectLight");

  return (
    <DetailSection title={"Indirect light"} description={"Light the surfaces bounce onto each other."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={RENDER_INDIRECT_LIGHT_SETTINGS.mode.label}
          description={explainRenderIndirectLightMode(indirectLight.mode)}
          isChecked={indirectLight.mode === ERenderIndirectLightMode.ENHANCED}
          onChange={(isOn: boolean) =>
            onSet({ mode: isOn ? ERenderIndirectLightMode.ENHANCED : ERenderIndirectLightMode.ENGINE })
          }
        />

        <RenderSettingRows
          settings={RENDER_INDIRECT_LIGHT_SETTINGS}
          names={["intensity", "radius", "steps"]}
          group={indirectLight}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
