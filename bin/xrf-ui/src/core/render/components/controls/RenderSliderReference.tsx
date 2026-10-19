import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useMemo } from "react";

import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { describeRenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { ISliderReferenceSource, SliderReferenceContext } from "@/core/ui/form/ReferenceSlider";

export interface IRenderSliderReferenceProps {
  /**
   * What the sliders within are measured by: the settings every viewport draws with, which a view's popover resets
   * to, or the preset they follow, which the settings reset to.
   */
  source: "settings" | "preset";
  children: ReactNode;
}

/**
 * Gives the feature sliders within their reference: the value at each one's place among the feature settings, marked
 * on its track and offered as its reset.
 */
export function RenderSliderReference({ source, children }: IRenderSliderReferenceProps): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const { preset } = renderSettingsService.rendererChoice;
  const features: IRenderFeatureSettings =
    source === "settings" ? renderSettingsService.rendererFeatures : RENDER_PRESETS[preset];
  const label: string = source === "settings" ? "Settings" : `${describeRenderPreset(preset)} preset`;

  const value: ISliderReferenceSource = useMemo(
    () => ({ label, resolve: (path: ReadonlyArray<string>) => readNumberAt(features, path) }),
    [features, label]
  );

  return <SliderReferenceContext.Provider value={value}>{children}</SliderReferenceContext.Provider>;
}

/** The number at a place in nested settings, or none where something else or nothing stands there. */
function readNumberAt(settings: object, path: ReadonlyArray<string>): Nullable<number> {
  let node: unknown = settings;

  for (const key of path) {
    node = typeof node === "object" && node !== null ? (node as Record<string, unknown>)[key] : undefined;
  }

  return typeof node === "number" && Number.isFinite(node) ? node : null;
}
