import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_ENHANCED_BLOOM_SETTINGS } from "@/core/render/lib/features/tone/render-bloom-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The glow around the frame's bright parts: the game's bloom, or the enhanced bloom in its place and its strengths. */
export function SettingsRendererBloom(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("enhancedBloom");

  return (
    <DetailSection title={"Bloom"} description={"The glow around the frame's bright parts."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_ENHANCED_BLOOM_SETTINGS}
          names={["mode", "threshold", "exposure", "blur", "vibrance", "sky"]}
          group={features.enhancedBloom}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
