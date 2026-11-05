import { useInjection } from "@wirestate/react";
import { Optional } from "@xrf/types";
import { useCallback } from "react";

import {
  TLevelFeatureGroupKey,
  useLevelFeatureOverride,
} from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { ILevelFeatureOptions, isLevelFeatureSet } from "@/core/level/lib/features/level-feature-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/** What a reset of the view's own values says it does: the view follows Settings › Rendering again. */
export const LEVEL_SETTINGS_RESET_LABEL: string = "Back to the settings";

/** What a reset of a value the game sets says it does: it stands as the game has it again. */
export const LEVEL_GAME_RESET_LABEL: string = "Reset to the game's";

/**
 * @param key - A feature group the view sets part of, as a popover section heads it.
 * @param unset - What the view keeps of the group once reset, where the popover owns only part of it (the Grass
 *   popover keeps the foliage's motion, the Wind popover's); the whole group goes back to the settings by default.
 * @returns What gives the group back to the settings, for the section's heading, while that changes what is drawn;
 *   none while what the view sets draws as the settings do.
 */
export function useLevelFeatureReset<K extends TLevelFeatureGroupKey & keyof IRenderFeatureSettings>(
  key: K,
  unset?: ILevelFeatureOptions[K]
): Optional<() => void> {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings, features } = useLevelRenderFeatures();
  const { reset } = useLevelFeatureOverride(key);

  const resetPart = useCallback(
    () => viewService.setFeatures({ ...viewService.features, [key]: unset }),
    [viewService, key, unset]
  );

  if (!isLevelFeatureSet(key, settings, features, unset)) {
    return undefined;
  }

  return unset === undefined ? reset : resetPart;
}
