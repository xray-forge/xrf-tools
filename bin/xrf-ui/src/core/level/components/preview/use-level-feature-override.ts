import { useInjection } from "@wirestate/react";
import { useCallback, useMemo } from "react";

import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { LevelViewService } from "@/core/level/services/level-view.service";

/** A group of features the view sets part of over the settings, rather than one value. */
export type TLevelFeatureGroupKey = {
  [K in keyof ILevelFeatureOptions]: ILevelFeatureOptions[K] extends object ? K : never;
}[keyof ILevelFeatureOptions];

/** A view's override of one feature group. */
export interface ILevelFeatureOverride<K extends TLevelFeatureGroupKey> {
  /** Sets part of the group for the view, the rest as it was. */
  set: (part: ILevelFeatureOptions[K]) => void;
  /** Leaves the whole group to the settings again. */
  reset: () => void;
}

/**
 * @param key - The feature group.
 * @returns How the group's popover changes it for the view.
 */
export function useLevelFeatureOverride<K extends TLevelFeatureGroupKey>(key: K): ILevelFeatureOverride<K> {
  const viewService: LevelViewService = useInjection(LevelViewService);

  const set = useCallback(
    (part: ILevelFeatureOptions[K]) =>
      viewService.setFeatures({ ...viewService.features, [key]: { ...viewService.features[key], ...part } }),
    [viewService, key]
  );

  const reset = useCallback(() => viewService.setFeatures({ ...viewService.features, [key]: {} }), [viewService, key]);

  return useMemo(() => ({ reset, set }), [reset, set]);
}
