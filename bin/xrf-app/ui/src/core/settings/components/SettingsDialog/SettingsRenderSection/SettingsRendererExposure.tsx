import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement, useCallback } from "react";

import { RenderSettingRow } from "@/core/render/components/settings/RenderSettingRow";
import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/features/tone/render-exposure-choices";
import { RENDER_GRADING_SETTINGS } from "@/core/render/lib/features/tone/render-grading-choices";
import { getRenderLightLevelOptions } from "@/core/render/lib/features/tone/render-light-level-choices";
import { RENDER_LIGHT_SETTINGS } from "@/core/render/lib/features/tone/render-light-model-choices";
import {
  IRenderFeatureSettings,
  TRenderLightSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/**
 * The frame's light and exposure: the light model and its levels, how the physical light's frame is graded and shown,
 * the engine's tonemap, and whether the exposure adapts to the frame's own luminance, towards what, and how fast.
 */
export function SettingsRendererExposure(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const onSetExposure = useRendererOverride("exposure");
  const onSetGrading = useRendererOverride("grading");
  const onSetLight = useCallback(
    (part: Partial<TRenderLightSettings>): void => renderSettingsService.setRendererOverrides(part),
    [renderSettingsService]
  );

  return (
    <DetailSection
      title={"Exposure"}
      description={"Automatic exposure: the engine tonemapper's meter, or the physical light's histogram."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRow
          settings={RENDER_LIGHT_SETTINGS}
          name={"lightModel"}
          group={features}
          features={features}
          onSet={onSetLight}
        />

        <RenderSettingRow
          settings={RENDER_LIGHT_SETTINGS}
          name={"lightLevel"}
          group={features}
          features={features}
          options={getRenderLightLevelOptions(renderSettingsService.isDevLightLevelAvailable)}
          onSet={onSetLight}
        />

        <RenderSettingRows
          settings={RENDER_LIGHT_SETTINGS}
          names={["sunTint", "displayTransform"]}
          group={features}
          features={features}
          onSet={onSetLight}
        />

        <RenderSettingRows
          settings={RENDER_GRADING_SETTINGS}
          names={["contrast", "saturation"]}
          group={features.grading}
          features={features}
          onSet={onSetGrading}
        />

        <RenderSettingRow
          settings={RENDER_LIGHT_SETTINGS}
          name={"tonemap"}
          group={features}
          features={features}
          onSet={onSetLight}
        />

        <RenderSettingRows
          settings={RENDER_EXPOSURE_SETTINGS}
          names={["isEnabled", "compensation", "middleGray", "amount", "lowLuminance", "adaptation"]}
          group={features.exposure}
          features={features}
          onSet={onSetExposure}
        />
      </div>
    </DetailSection>
  );
}
