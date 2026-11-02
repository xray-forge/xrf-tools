import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_GLASS_SETTINGS } from "@/core/render/lib/features/world/render-glass-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** How glass is drawn: blended over what is behind it as the game does, or the enhanced glass refracting it. */
export function SettingsRendererGlass(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("glass");

  return (
    <DetailSection title={"Glass"} description={"How windows and other glass are drawn over what is behind them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_GLASS_SETTINGS}
          names={["mode", "refraction"]}
          group={features.glass}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
