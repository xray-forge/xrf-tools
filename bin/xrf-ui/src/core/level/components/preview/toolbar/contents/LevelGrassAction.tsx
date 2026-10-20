import { default as GrassIcon } from "@mui/icons-material/Grass";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ERenderFoliageMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { describeLevelFeatureToggle } from "@/core/level/lib/features/level-feature-options";
import { LEVEL_FOLIAGE_LIGHT_KEYS, resetLevelFoliageKeys } from "@/core/level/lib/features/level-foliage-keys";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatFoliageStrength,
  formatGrassDensity,
  formatGrassHeight,
  formatGrassRadius,
  formatGrassShadowCascades,
  fromGrassDensityScale,
  RENDER_FOLIAGE_LIMITS,
  RENDER_GRASS_LIMITS,
  toGrassDensityScale,
} from "@/core/render/lib/features/render-grass-choices";
import { TRenderFoliageSettings, TRenderGrassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The light the enhanced foliage passes through leaves and grass, each a slider; its motion is the Wind popover's. */
const FOLIAGE_LIGHT: ReadonlyArray<[keyof typeof RENDER_FOLIAGE_LIMITS, string]> = [
  ["sssIntensity", "Light through leaves"],
  ["sssColor", "Sun colour through leaves"],
];

/**
 * Whether this view plants the level's grass, how dense, how far and how tall, whether it casts shadows, and the light
 * passing through leaves and grass.
 */
export function LevelGrassAction({
  "data-testid": dataTestId = "level-grass-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isGrassy");
  const { state, set, setNested } = useLevelFeature("grass");

  const grass: TRenderGrassSettings = state.value;
  const foliage: TRenderFoliageSettings = grass.foliage;

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
        label={"Density"}
        value={toGrassDensityScale(grass.density)}
        {...RENDER_GRASS_LIMITS.density}
        format={(scale: number) => formatGrassDensity(fromGrassDensityScale(scale))}
        onChange={(scale: number) => set({ density: fromGrassDensityScale(scale) })}
      />

      <RenderValueSlider
        label={"Radius"}
        value={grass.radius}
        {...RENDER_GRASS_LIMITS.radius}
        format={formatGrassRadius}
        onChange={(radius: number) => set({ radius })}
      />

      <RenderValueSlider
        label={"Height"}
        value={grass.height}
        {...RENDER_GRASS_LIMITS.height}
        format={formatGrassHeight}
        onChange={(height: number) => set({ height })}
      />

      <RenderValueSlider
        label={"Slope limit"}
        value={grass.slopeLimit}
        min={10}
        max={90}
        step={1}
        format={(value: number) => `${value.toFixed(0)}°`}
        onChange={(slopeLimit: number) => set({ slopeLimit })}
      />

      <CheckboxFormRow
        label={"Aligned to the ground"}
        isChecked={grass.isAligned}
        onChange={(isAligned: boolean) => set({ isAligned })}
      />

      <CheckboxFormRow
        label={"Shadows"}
        isChecked={grass.isShadowed}
        onChange={(isShadowed: boolean) => set({ isShadowed })}
      />

      {grass.isShadowed ? (
        <>
          <RenderValueSlider
            label={"Shadow radius"}
            value={grass.shadowRadius}
            {...RENDER_GRASS_LIMITS.shadowRadius}
            format={formatGrassRadius}
            onChange={(shadowRadius: number) => set({ shadowRadius })}
          />
          <RenderValueSlider
            label={"Shadow cascades"}
            value={grass.shadowCascades}
            {...RENDER_GRASS_LIMITS.shadowCascades}
            format={formatGrassShadowCascades}
            onChange={(shadowCascades: number) => set({ shadowCascades })}
          />
        </>
      ) : null}

      {foliage.mode === ERenderFoliageMode.ENHANCED ? (
        FOLIAGE_LIGHT.map(([key, label]) => (
          <RenderValueSlider
            key={key}
            label={label}
            value={foliage[key]}
            {...RENDER_FOLIAGE_LIMITS[key]}
            format={formatFoliageStrength}
            onChange={(value: number) => setFoliage({ [key]: value })}
          />
        ))
      ) : (
        <p className={"text-xs text-text-secondary"}>
          Light through leaves comes with the Enhanced foliage motion, under Wind.
        </p>
      )}

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverToggle>
  );
}
