import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_CONTACT_SHADOW_SETTINGS } from "@/core/render/lib/features/lighting/render-contact-shadow-choices";
import {
  IRenderFeatureSettings,
  TRenderContactShadowSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The small shadows the sun's maps miss, found in the frame's depth: whether they are, how long and how dark. */
export function SettingsRendererContactShadows(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("shadows");

  return (
    <DetailSection
      title={"Contact shadows"}
      description={"Shadows too small for the sun's maps, marched over the frame's depth."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_CONTACT_SHADOW_SETTINGS}
          names={["mode", "length", "intensity", "thickness", "steps", "isFullResolution", "lights"]}
          group={features.shadows.contact}
          features={features}
          onSet={(contact: Partial<TRenderContactShadowSettings>) => onSet({ contact })}
        />
      </div>
    </DetailSection>
  );
}
