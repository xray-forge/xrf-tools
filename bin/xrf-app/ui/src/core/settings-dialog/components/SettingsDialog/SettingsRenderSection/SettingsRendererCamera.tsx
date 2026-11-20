import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_CAMERA_SETTINGS } from "@/core/render/lib/features/image/render-camera-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/**
 * The camera the frame is shown through: motion blur, depth of field, vignette, film grain and chromatic aberration,
 * each off until turned on, and what shapes it.
 */
export function SettingsRendererCamera(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const onSet = useRendererOverride("camera");
  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  return (
    <DetailSection
      title={"Camera"}
      description={"A physical camera's effects over the finished frame, each off until turned on."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_CAMERA_SETTINGS}
          names={[
            "isMotionBlurred",
            "shutter",
            "motionLimit",
            "isDefocused",
            "fStop",
            "focusDistance",
            "sensorHeight",
            "isVignetted",
            "vignetteStrength",
            "isGrained",
            "grainStrength",
            "grainSize",
            "isAberrated",
            "aberrationStrength",
          ]}
          group={features.camera}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
