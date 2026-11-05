import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_TERRAIN_SETTINGS } from "@/core/render/lib/features/world/render-terrain-choices";
import { TRenderTerrainSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * How this view's varied terrain tiling lays its hexagons: how small, how far their copies turn, how sharply they meet.
 * Their reset is the section's heading's.
 */
export function LevelTerrainSettings({
  "data-testid": dataTestId = "level-terrain-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride("terrain");

  const value: TRenderTerrainSettings = toLevelModeFeature("terrain", settings, features);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderSettingEntries
        settings={RENDER_TERRAIN_SETTINGS}
        names={["scale", "rotation", "sharpness"]}
        group={value}
        features={drawn}
        onSet={set}
      />
    </div>
  );
}
