import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_FOG_SETTINGS } from "@/core/render/lib/features/world/render-fog-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** How the fog is drawn beyond the weather's distance and colour: the game's, or the enhanced fog and its strengths. */
export function SettingsRendererFog(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("fog");

  return (
    <DetailSection title={"Fog"} description={"How the weather's fog is drawn: the game's, or thickened low down."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_FOG_SETTINGS}
          names={["mode", "height", "density", "sunColor", "scattering", "skyFade", "colour"]}
          group={features.fog}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
