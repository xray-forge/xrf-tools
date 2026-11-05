import { Optional } from "@xrf/types";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { LEVEL_SETTINGS_RESET_LABEL } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelFeatureReset } from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import {
  ILevelEnhancedFeature,
  LEVEL_ENHANCED_FEATURES,
  TLevelEnhancedFeatureKey,
} from "@/core/level/lib/features/level-enhanced-features";
import { ILevelFeatureOptions, toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

export interface ILevelEnhancedFeatureSectionProps<K extends TLevelEnhancedFeatureKey> extends BaseComponentProps {
  feature: K;
}

/**
 * A feature the view draws as the engine does or enhanced (`LEVEL_ENHANCED_FEATURES`): a toggle for the enhanced one
 * with what the mode drawn does, and the values tuning it as their declarations say, disabled while the engine's is
 * drawn.
 */
export function LevelEnhancedFeatureSection<K extends TLevelEnhancedFeatureKey>({
  "data-testid": dataTestId = "level-enhanced-feature-section",
  id,
  className,
  feature,
}: ILevelEnhancedFeatureSectionProps<K>): ReactElement {
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride(feature);
  const onReset: Optional<() => void> = useLevelFeatureReset(feature);

  const enhanced: ILevelEnhancedFeature<K> = LEVEL_ENHANCED_FEATURES[feature];
  const value: IRenderFeatureSettings[K] = toLevelModeFeature(feature, settings, features);
  // Every mode of these is the engine's or the enhanced, alike by name.
  const isEnhanced: boolean = value.mode === "enhanced";

  return (
    <EditorPopoverGroupSection
      data-testid={dataTestId}
      id={id}
      className={className}
      label={enhanced.label}
      note={enhanced.explain(value.mode)}
      isOn={isEnhanced}
      resetLabel={LEVEL_SETTINGS_RESET_LABEL}
      onToggle={() => set({ mode: isEnhanced ? "engine" : "enhanced" } as ILevelFeatureOptions[K])}
      onReset={onReset}
    >
      <RenderSettingEntries
        settings={enhanced.settings}
        names={enhanced.names}
        group={value}
        features={drawn}
        onSet={(part: Partial<IRenderFeatureSettings[K]>) => set(part as ILevelFeatureOptions[K])}
      />
    </EditorPopoverGroupSection>
  );
}
