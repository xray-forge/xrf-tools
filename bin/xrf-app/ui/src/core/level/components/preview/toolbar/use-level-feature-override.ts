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
  /**
   * Sets part of one of the group's nested settings for the view, the rest of it left to what the view or the settings
   * had, so a later change to the settings still reaches what the view never set.
   */
  setNested: <G extends keyof Required<ILevelFeatureOptions[K]>>(
    group: G,
    part: Partial<NonNullable<Required<ILevelFeatureOptions[K]>[G]>>
  ) => void;
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

  const setNested = useCallback(
    <G extends keyof Required<ILevelFeatureOptions[K]>>(
      group: G,
      part: Partial<NonNullable<Required<ILevelFeatureOptions[K]>[G]>>
    ) => {
      const own: Required<ILevelFeatureOptions[K]> = viewService.features[key] as Required<ILevelFeatureOptions[K]>;

      viewService.setFeatures({
        ...viewService.features,
        [key]: { ...own, [group]: { ...(own[group] ?? {}), ...part } },
      });
    },
    [viewService, key]
  );

  const reset = useCallback(() => viewService.resetFeatures([key]), [viewService, key]);

  return useMemo(() => ({ reset, set, setNested }), [reset, set, setNested]);
}
