import { useInjection } from "@wirestate/react";
import { ERenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { TextureIcon } from "@xrf/ui/icons/texture";
import { ReactElement } from "react";

import { LEVEL_SETTINGS_RESET_LABEL } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import {
  ILevelFeatureOverride,
  useLevelFeatureOverride,
} from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { LevelParallaxSettings } from "@/core/level/components/preview/toolbar/view/LevelParallaxSettings";
import { LevelTerrainSettings } from "@/core/level/components/preview/toolbar/view/LevelTerrainSettings";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { isRenderParallaxDrawn } from "@/core/render/lib/features/world/render-parallax-choices";
import { explainRenderTerrainTiling } from "@/core/render/lib/features/world/render-terrain-choices";
import {
  TRenderParallaxSettings,
  TRenderTerrainSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * The surfaces' relief: their bumps, their parallax with its settings, and the terrain's details varied or repeating as
 * a grid with theirs. What the frame shows, the wireframe and the wall marks are no part of it: they have no settings to
 * tune beside their switch.
 */
export function LevelReliefAction({
  "data-testid": dataTestId = "level-relief-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings, features } = useLevelRenderFeatures();
  const parallaxOverride: ILevelFeatureOverride<"parallax"> = useLevelFeatureOverride("parallax");
  const terrainOverride: ILevelFeatureOverride<"terrain"> = useLevelFeatureOverride("terrain");

  const options: ILevelViewOptions = viewService.options;
  const parallax: TRenderParallaxSettings = toLevelModeFeature("parallax", settings, features);
  const isParallaxAvailable: boolean = isRenderParallaxDrawn(parallax);
  const isParallaxed: boolean = options.isParallaxed && isParallaxAvailable;
  const terrain: TRenderTerrainSettings = toLevelModeFeature("terrain", settings, features);
  const isTerrainVaried: boolean = terrain.tiling === ERenderTerrainTiling.VARIED;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Relief"}
      description={[
        options.isBumped ? "Bumps" : "Flat",
        isParallaxed || !isParallaxAvailable ? null : "without parallax",
        isTerrainVaried ? "varied terrain" : null,
      ]
        .filter((it): it is string => it !== null)
        .join(", ")}
      icon={<TextureIcon />}
      isActive={!options.isBumped || (isParallaxAvailable && !isParallaxed)}
    >
      <EditorPopoverGroupSection
        label={"Bumps"}
        description={"Shades each surface with its textures' bump maps."}
        isOn={options.isBumped}
        onToggle={() => viewService.toggleOption("isBumped")}
      />

      <EditorPopoverGroupSection
        label={"Parallax"}
        description={
          isParallaxAvailable
            ? "Displaces each surface by its textures' height."
            : "The settings draw neither the steep nor the enhanced parallax."
        }
        isOn={isParallaxed}
        isDisabled={!isParallaxAvailable}
        resetLabel={LEVEL_SETTINGS_RESET_LABEL}
        onToggle={() => viewService.toggleOption("isParallaxed")}
        onReset={parallaxOverride.isSet ? parallaxOverride.reset : undefined}
      >
        <LevelParallaxSettings />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        data-testid={"level-terrain-tiling"}
        label={"Varied terrain"}
        description={explainRenderTerrainTiling(ERenderTerrainTiling.VARIED)}
        isOn={isTerrainVaried}
        resetLabel={LEVEL_SETTINGS_RESET_LABEL}
        onToggle={() =>
          terrainOverride.set({ tiling: isTerrainVaried ? ERenderTerrainTiling.ENGINE : ERenderTerrainTiling.VARIED })
        }
        onReset={terrainOverride.isSet ? terrainOverride.reset : undefined}
      >
        <LevelTerrainSettings />
      </EditorPopoverGroupSection>
    </EditorPopoverGroup>
  );
}
