import { ERenderFoliageMode } from "@xrf/ipc/types/xrf-renderer";
import { Optional } from "@xrf/types";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { GrassIcon } from "@xrf/ui/icons/grass";
import { ReactElement, useCallback } from "react";

import { LEVEL_SETTINGS_RESET_LABEL } from "@/core/level/components/preview/toolbar/LevelSettingsResetButton";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelFeatureReset } from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { LEVEL_FOLIAGE_LIGHT_KEYS, resetLevelFoliageKeys } from "@/core/level/lib/features/level-foliage-keys";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RenderSettingSection } from "@/core/render/components/settings/RenderSettingSection";
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
 * Whether this view plants the level's grass, how dense, how far and how tall; its shadows; and the light passing
 * through leaves and grass. The foliage's motion is the Wind popover's, which this popover's reset leaves alone.
 */
export function LevelGrassAction({
  "data-testid": dataTestId = "level-grass-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isGrassy");
  const { state, set, setNested } = useLevelFeature("grass");
  const onReset: Optional<() => void> = useLevelFeatureReset(
    "grass",
    resetLevelFoliageKeys({ foliage: features.grass.foliage }, settings.grass.foliage, LEVEL_FOLIAGE_LIGHT_KEYS)
  );

  const grass: TRenderGrassSettings = state.value;
  const isEnhancedFoliage: boolean = grass.foliage.mode === ERenderFoliageMode.ENHANCED;

  const setFoliage = useCallback(
    (part: Partial<TRenderFoliageSettings>): void => setNested("foliage", part),
    [setNested]
  );

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
      resetLabel={LEVEL_SETTINGS_RESET_LABEL}
      onToggle={toggle}
      onReset={onReset}
    >
      <RenderValueSlider
        label={RENDER_GRASS_SETTINGS.density.label}
        description={RENDER_GRASS_SETTINGS.density.description as string}
        value={toGrassDensityScale(grass.density)}
        {...RENDER_GRASS_DENSITY_LIMITS}
        format={(scale: number) => formatGrassDensity(fromGrassDensityScale(scale))}
        onChange={(scale: number) => set({ density: fromGrassDensityScale(scale) })}
      />

      <RenderSettingEntries
        settings={RENDER_GRASS_SETTINGS}
        names={["radius", "height", "slopeLimit", "isAligned"]}
        group={grass}
        features={drawn}
        onSet={set}
      />

      <RenderSettingSection
        settings={RENDER_GRASS_SETTINGS}
        name={"isShadowed"}
        names={["shadowRadius", "shadowCascades"]}
        group={grass}
        features={drawn}
        onSet={set}
      />

      <EditorPopoverGroupSection
        label={"Light through leaves"}
        note={isEnhancedFoliage ? undefined : "Comes with the Enhanced foliage motion, under Wind."}
      >
        <RenderSettingEntries
          settings={RENDER_FOLIAGE_SETTINGS}
          names={LEVEL_FOLIAGE_LIGHT_KEYS}
          group={grass.foliage}
          features={drawn}
          onSet={setFoliage}
        />
      </EditorPopoverGroupSection>
    </EditorPopoverToggle>
  );
}
