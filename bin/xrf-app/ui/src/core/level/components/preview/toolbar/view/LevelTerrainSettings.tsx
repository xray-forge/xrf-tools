import { ERenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatTerrainRotation,
  formatTerrainScale,
  formatTerrainSharpness,
  RENDER_TERRAIN_LIMITS,
} from "@/core/render/lib/features/render-terrain-choices";
import { TRenderTerrainSettings } from "@/core/render/lib/settings/render-feature-settings";

/** How this view's varied terrain tiling lays its hexagons: how small, how far their copies turn, how sharply they meet. */
export function LevelTerrainSettings({
  "data-testid": dataTestId = "level-terrain-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("terrain");

  const value: TRenderTerrainSettings = toLevelModeFeature("terrain", settings, features);
  const isEngine: boolean = value.tiling === ERenderTerrainTiling.ENGINE;

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueSlider
        label={"Scale"}
        value={value.scale}
        {...RENDER_TERRAIN_LIMITS.scale}
        isDisabled={isEngine}
        format={formatTerrainScale}
        onChange={(scale: number) => set({ scale })}
      />

      <RenderValueSlider
        label={"Rotation"}
        value={value.rotation}
        {...RENDER_TERRAIN_LIMITS.rotation}
        isDisabled={isEngine}
        format={formatTerrainRotation}
        onChange={(rotation: number) => set({ rotation })}
      />

      <RenderValueSlider
        label={"Blend sharpness"}
        value={value.sharpness}
        {...RENDER_TERRAIN_LIMITS.sharpness}
        isDisabled={isEngine}
        format={formatTerrainSharpness}
        onChange={(sharpness: number) => set({ sharpness })}
      />

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </div>
  );
}
