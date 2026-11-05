import { useInjection } from "@wirestate/react";
import { ERenderFoliageMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverNote } from "@xrf/ui/editor/popover-row";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { GrassIcon } from "@xrf/ui/icons/grass";
import { ReactElement, useCallback } from "react";

import { LevelSettingsResetButton } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { LEVEL_FOLIAGE_LIGHT_KEYS, resetLevelFoliageKeys } from "@/core/level/lib/features/level-foliage-keys";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import {
  formatGrassDensity,
  formatGrassRadius,
  fromGrassDensityScale,
  RENDER_FOLIAGE_SETTINGS,
  RENDER_GRASS_DENSITY_LIMITS,
  RENDER_GRASS_SETTINGS,
  toGrassDensityScale,
} from "@/core/render/lib/features/world/render-grass-choices";
import {
  TRenderFoliageSettings,
  TRenderGrassSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * Whether this view plants the level's grass, how dense, how far and how tall, whether it casts shadows, and the light
 * passing through leaves and grass; the foliage's motion is the Wind popover's.
 */
export function LevelGrassAction({
  "data-testid": dataTestId = "level-grass-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings, drawn } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isGrassy");
  const { state, set, setNested } = useLevelFeature("grass");

  const grass: TRenderGrassSettings = state.value;
  const isEnhancedFoliage: boolean = grass.foliage.mode === ERenderFoliageMode.ENHANCED;

  const setFoliage = useCallback(
    (part: Partial<TRenderFoliageSettings>): void => setNested("foliage", part),
    [setNested]
  );

  // The foliage's motion is the Wind popover's, which a reset here leaves alone.
  const reset = useCallback(() => {
    const grassOptions = viewService.features.grass;

    viewService.setFeatures({
      ...viewService.features,
      grass: resetLevelFoliageKeys({ foliage: grassOptions.foliage }, settings.grass.foliage, LEVEL_FOLIAGE_LIGHT_KEYS),
    });
  }, [viewService, settings]);

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Grass"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        label: "Grass",
        off: "Grass off, the ground bare",
        on: `Grass to ${formatGrassRadius(grass.radius)}, ${formatGrassDensity(grass.density)} the game's density`,
      })}
      icon={<GrassIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Grass"}
      onToggle={toggle}
    >
      <RenderValueSlider
        label={RENDER_GRASS_SETTINGS.density.label}
        value={toGrassDensityScale(grass.density)}
        {...RENDER_GRASS_DENSITY_LIMITS}
        format={(scale: number) => formatGrassDensity(fromGrassDensityScale(scale))}
        onChange={(scale: number) => set({ density: fromGrassDensityScale(scale) })}
      />

      <RenderSettingEntries
        settings={RENDER_GRASS_SETTINGS}
        names={["radius", "height", "slopeLimit", "isAligned", "isShadowed", "shadowRadius", "shadowCascades"]}
        group={grass}
        features={drawn}
        onSet={set}
      />

      {isEnhancedFoliage ? null : (
        <EditorPopoverNote>Light through leaves comes with the Enhanced foliage motion, under Wind.</EditorPopoverNote>
      )}

      <RenderSettingEntries
        settings={RENDER_FOLIAGE_SETTINGS}
        names={LEVEL_FOLIAGE_LIGHT_KEYS}
        group={grass.foliage}
        features={drawn}
        onSet={setFoliage}
      />

      <LevelSettingsResetButton onReset={reset} />
    </EditorPopoverToggle>
  );
}
