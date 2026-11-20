import { useInjection } from "@wirestate/react";
import { ERenderDebandingMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  explainRenderDebandingMode,
  RENDER_DEBANDING_SETTINGS,
} from "@/core/render/lib/features/image/render-debanding-choices";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** Whether the sky's colour bands are smoothed. */
export function SettingsRendererDebanding(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const debanding: TRenderDebandingSettings = renderSettingsService.rendererFeatures.debanding;

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
      </div>
    </DetailSection>
  );
}
