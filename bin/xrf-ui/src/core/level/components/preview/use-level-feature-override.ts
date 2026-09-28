import { useCallback, useMemo } from "react";

import { ILevelFeatureOptions, TLevelFeatureKey } from "@/core/level/lib/features/level-feature-options";

/** A view's override of one feature group. */
export interface ILevelFeatureOverride<K extends TLevelFeatureKey> {
  /** Sets part of the group for the view, the rest as it was. */
  set: (part: ILevelFeatureOptions[K]) => void;
  /** Leaves the whole group to the settings again. */
  reset: () => void;
}

/**
 * @param key - The feature group.
 * @param features - What the view sets over the settings.
 * @param onChange - Told what it sets from now on.
 * @returns How the group's popover changes it.
 */
export function useLevelFeatureOverride<K extends TLevelFeatureKey>(
  key: K,
  features: ILevelFeatureOptions,
  onChange: (features: ILevelFeatureOptions) => void
): ILevelFeatureOverride<K> {
  const set = useCallback(
    (part: ILevelFeatureOptions[K]) => onChange({ ...features, [key]: { ...features[key], ...part } }),
    [key, features, onChange]
  );

  const reset = useCallback(() => onChange({ ...features, [key]: {} }), [key, features, onChange]);

  return useMemo(() => ({ reset, set }), [reset, set]);
}
