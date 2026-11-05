import { Optional } from "@xrf/types";

import {
  TLevelFeatureGroupKey,
  useLevelFeatureOverride,
} from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { isLevelFeatureSet } from "@/core/level/lib/features/level-feature-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * @param key - A feature group the view sets part of, as a popover section heads it.
 * @returns What gives the group back to the settings, for the section's heading, while that changes what is drawn;
 *   none while what the view sets draws as the settings do.
 */
export function useLevelFeatureReset<K extends TLevelFeatureGroupKey & keyof IRenderFeatureSettings>(
  key: K
): Optional<() => void> {
  const { settings, features } = useLevelRenderFeatures();
  const { reset } = useLevelFeatureOverride(key);

  return isLevelFeatureSet(key, settings, features) ? reset : undefined;
}
