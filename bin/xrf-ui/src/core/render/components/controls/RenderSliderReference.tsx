import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useMemo } from "react";

import { readRenderFeatureNumber, TRenderFeaturePath } from "@/core/render/lib/settings/render-feature-path";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { describeRenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { ISliderReferenceSource, SliderReferenceContext } from "@/core/ui/form/ReferenceSlider";

export interface IRenderSliderReferenceProps {
  /** The settings every viewport draws with, which a view's popover resets to, or the preset they follow. */
  source: "settings" | "preset";
  children: ReactNode;
}

/**
 * Gives the feature sliders within their reference: the value at each one's place among the feature settings.
 */
export function RenderSliderReference({ source, children }: IRenderSliderReferenceProps): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const { preset } = renderSettingsService.rendererChoice;
  const features: IRenderFeatureSettings =
    source === "settings" ? renderSettingsService.rendererFeatures : RENDER_PRESETS[preset];
  const label: string = source === "settings" ? "Settings" : `${describeRenderPreset(preset)} preset`;

  const value: ISliderReferenceSource = useMemo(
    () => ({ label, resolve: (path: TRenderFeaturePath) => readRenderFeatureNumber(features, path) }),
    [features, label]
  );

  return <SliderReferenceContext.Provider value={value}>{children}</SliderReferenceContext.Provider>;
}
