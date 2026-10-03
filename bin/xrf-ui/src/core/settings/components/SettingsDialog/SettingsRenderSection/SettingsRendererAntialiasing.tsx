import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ERendererAntialiasing } from "@/core/render/lib/contract/renderer-antialiasing";
import { IRendererFeatureSettings } from "@/core/render/lib/contract/renderer-feature-settings";
import { ERendererRenderScale } from "@/core/render/lib/contract/renderer-render-scale";
import {
  formatSharpening,
  RENDER_ANTIALIASING_OPTIONS,
  RENDER_SCALE_OPTIONS,
  RENDER_SHARPENING_LIMITS,
} from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

/** How the frame's edges are smoothed, and how much of it is drawn before it is upscaled to the view. */
export function SettingsRendererAntialiasing(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const features: IRendererFeatureSettings = settingsService.rendererFeatures;

  return (
    <DetailSection title={"Antialiasing and upscaling"} description={"Edge smoothing and internal render scale."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Antialiasing"}
          description={
            "SMAA: sharp and stable. FXAA: cheaper, softer. TAA: temporal, also smooths foliage and thin geometry. " +
            "FSR 2: AMD's temporal upscaler, steadiest on fine detail."
          }
          options={RENDER_ANTIALIASING_OPTIONS}
          value={features.antialiasing}
          onChange={(antialiasing: ERendererAntialiasing) => settingsService.setRendererOverrides({ antialiasing })}
        />

        <ChoiceFormRow
          label={"Render scale"}
          description={
            "Share of the output resolution the scene renders at. TAA and FSR 2 upscale temporally; other modes " +
            "use FSR 1."
          }
          options={RENDER_SCALE_OPTIONS}
          value={features.upscaling.scale}
          onChange={(scale: ERendererRenderScale) => settingsService.setRendererOverrides({ upscaling: { scale } })}
        />

        <SliderFormRow
          label={"Sharpening"}
          description={"RCAS sharpening after upscaling. Inactive at native scale."}
          value={features.upscaling.sharpening}
          {...RENDER_SHARPENING_LIMITS}
          isDisabled={features.upscaling.scale === ERendererRenderScale.NATIVE}
          format={formatSharpening}
          onChange={(sharpening: number) => settingsService.setRendererOverrides({ upscaling: { sharpening } })}
        />
      </div>
    </DetailSection>
  );
}
