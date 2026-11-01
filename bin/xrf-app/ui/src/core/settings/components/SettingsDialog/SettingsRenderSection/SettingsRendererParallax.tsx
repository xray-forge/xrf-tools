import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_PARALLAX_SETTINGS } from "@/core/render/lib/features/world/render-parallax-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** How the level's surfaces are displaced by their height: the engines' steep parallax, or the enhanced terrain march. */
export function SettingsRendererParallax(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("parallax");

  return (
    <DetailSection title={"Parallax"} description={"Displaces surfaces by their height."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_PARALLAX_SETTINGS}
          names={[
            "mode",
            "isSteep",
            "steps",
            "range",
            "height",
            "surfaceHeight",
            "surfaceRange",
            "isRefined",
            "shadow",
            "blending",
            "cavity",
          ]}
          group={features.parallax}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
