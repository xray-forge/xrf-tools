import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material";
import { EditorPopoverGroupSection } from "@xrf/ui/shell/editor/EditorPopoverGroup";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import {
  ILevelEnhancedFeature,
  ILevelEnhancedFeatureSlider,
  LEVEL_ENHANCED_FEATURES,
  TLevelEnhancedFeatureKey,
} from "@/core/level/lib/features/level-enhanced-features";
import { ILevelFeatureOptions, toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

export interface ILevelEnhancedFeatureSectionProps<K extends TLevelEnhancedFeatureKey> extends BaseComponentProps {
  feature: K;
}

/**
 * A feature the view draws as the engine does or enhanced (`LEVEL_ENHANCED_FEATURES`): a toggle for the enhanced one
 * with what the mode drawn does, and the values tuning it, shown but disabled while the engine's is drawn.
 */
export function LevelEnhancedFeatureSection<K extends TLevelEnhancedFeatureKey>({
  "data-testid": dataTestId = "level-enhanced-feature-section",
  id,
  className,
  feature,
}: ILevelEnhancedFeatureSectionProps<K>): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride(feature);

  const { label, subject, explain, sliders }: ILevelEnhancedFeature<K> = LEVEL_ENHANCED_FEATURES[feature];
  const value: IRenderFeatureSettings[K] = toLevelModeFeature(feature, settings, features);
  // Every mode of these is the engine's or the enhanced, alike by name.
  const isEnhanced: boolean = value.mode === "enhanced";

  return (
    <EditorPopoverGroupSection
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={explain(value.mode)}
      isOn={isEnhanced}
      onToggle={() => set({ mode: isEnhanced ? "engine" : "enhanced" } as ILevelFeatureOptions[K])}
    >
      {sliders.map((slider: ILevelEnhancedFeatureSlider<K>) => (
        <RenderValueSlider
          key={String(slider.key)}
          label={slider.label}
          value={value[slider.key] as number}
          {...slider.limits}
          isDisabled={!isEnhanced}
          format={slider.format}
          onChange={(it: number) => set({ [slider.key]: it } as ILevelFeatureOptions[K])}
        />
      ))}

      <Button size={"small"} aria-label={`Reset the ${subject} to the preset`} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverGroupSection>
  );
}
