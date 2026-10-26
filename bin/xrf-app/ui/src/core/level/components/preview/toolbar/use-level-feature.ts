import {
  ILevelFeatureOverride,
  useLevelFeatureOverride,
} from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import {
  ILevelFeatureState,
  TLevelFeatureKey,
  toLevelFeatureState,
} from "@/core/level/lib/features/level-feature-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** A feature group behind a toolbar toggle: as the view draws it while on, and how the view changes it. */
export interface ILevelFeature<K extends TLevelFeatureKey> extends ILevelFeatureOverride<K> {
  state: ILevelFeatureState<IRenderFeatureSettings[K]>;
}

/**
 * @param key - The feature group.
 * @returns The group as the view draws it while on, whether the settings let it be, and how to change it.
 */
export function useLevelFeature<K extends TLevelFeatureKey>(key: K): ILevelFeature<K> {
  const { settings, features } = useLevelRenderFeatures();
  const { set, setNested, reset } = useLevelFeatureOverride(key);

  return { reset, set, setNested, state: toLevelFeatureState(key, settings, features) };
}
