import { useInjection } from "@wirestate/react";
import { ERenderDebandingMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  explainRenderDebandingMode,
  RENDER_DEBANDING_SETTINGS,
} from "@/core/render/lib/features/image/render-debanding-choices";
import {
  IRenderFeatureSettings,
  TRenderDebandingSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** Whether the sky's colour bands are smoothed, at what quality and over how many pixels. */
export function SettingsRendererDebanding(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const debanding: TRenderDebandingSettings = features.debanding;

  const onSet = useRendererOverride("debanding");

  return (
    <DetailSection title={"Debanding"} description={"Smooths the colour bands in the sky's gradients."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={RENDER_DEBANDING_SETTINGS.mode.label}
          description={explainRenderDebandingMode(debanding.mode)}
          isChecked={debanding.mode === ERenderDebandingMode.ENHANCED}
          onChange={(isOn: boolean) =>
            onSet({ mode: isOn ? ERenderDebandingMode.ENHANCED : ERenderDebandingMode.ENGINE })
          }
        />

        <RenderSettingRows
          settings={RENDER_DEBANDING_SETTINGS}
          names={["passes", "radius"]}
          group={debanding}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
