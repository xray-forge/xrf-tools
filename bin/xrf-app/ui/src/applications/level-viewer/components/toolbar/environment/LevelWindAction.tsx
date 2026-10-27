import { useInjection } from "@wirestate/react";
import { formatDegrees, formatNumber, formatPercent } from "@xrf/format";
import { ERenderFoliageMode, RenderFoliageMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AirIcon } from "@xrf/ui/icons/air";
import { Button } from "@xrf/ui/material/button";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback } from "react";

import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LEVEL_FOLIAGE_MOTION_KEYS, resetLevelFoliageKeys } from "@/core/level/lib/features/level-foliage-keys";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_WIND_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderFoliageMode,
  formatFoliageStrength,
  RENDER_FOLIAGE_LIMITS,
  RENDER_FOLIAGE_MODE_OPTIONS,
} from "@/core/render/lib/features/render-grass-choices";
import { TRenderFoliageSettings } from "@/core/render/lib/settings/render-feature-settings";

/** The enhanced foliage motion's strengths, each a slider. */
const FOLIAGE_MOTION: ReadonlyArray<[keyof typeof RENDER_FOLIAGE_LIMITS, string]> = [
  ["grassSpeed", "Grass speed"],
  ["grassTurbulence", "Grass turbulence"],
  ["grassPush", "Grass push"],
  ["grassWave", "Grass wave"],
  ["treesSpeed", "Branches speed"],
  ["treesTrunk", "Trunk speed"],
  ["treesBend", "Trunk bend"],
];

/**
 * The wind the rain drifts on and the trees' sway; whether the trees and the grass sway at all, and how they move.
 */
export function LevelWindAction({
  "data-testid": dataTestId = "level-wind-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const { settings } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isWindy");
  const { state, setNested } = useLevelFeature("grass");

  const manual: ILevelManualWeather = weatherService.shown;
  const foliage: TRenderFoliageSettings = state.value.foliage;
  const isEnhancedFoliage: boolean = foliage.mode === ERenderFoliageMode.ENHANCED;

  const setFoliage = useCallback(
    (part: Partial<TRenderFoliageSettings>): void => setNested("foliage", part),
    [setNested]
  );

  // The light through leaves is the Grass popover's, which a reset here leaves alone.
  const resetFoliage = useCallback(() => {
    viewService.setFeatures({
      ...viewService.features,
      grass: resetLevelFoliageKeys(viewService.features.grass, settings.grass.foliage, LEVEL_FOLIAGE_MOTION_KEYS),
    });
  }, [viewService, settings]);

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Wind"}
      description={
        isOn
          ? `Wind ${formatNumber(manual.windVelocity, 1)} m/s, trees sway ${formatNumber(manual.treesAmplitude, 3)}`
          : "Wind off, trees stand still"
      }
      icon={<AirIcon />}
      isOn={isOn}
      toggleLabel={"Wind"}
      onToggle={toggle}
    >
      <LevelManualWeatherSlider
        field={"windVelocity"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 1)} m/s`}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"windDirection"}
        manual={manual}
        format={formatDegrees}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"treesAmplitude"}
        manual={manual}
        format={(value: number) => formatNumber(value, 3)}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"treesSpeed"}
        manual={manual}
        format={(value: number) => formatNumber(value, 2)}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"treesRotation"}
        manual={manual}
        format={formatDegrees}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherVectorField field={"treesWave"} manual={manual} onEdit={weatherService.editManual} />

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_WIND_KEYS} onReset={weatherService.resetManual} />

      <Typography className={"text-text-secondary"} variant={"overline"}>
        Foliage motion
      </Typography>

      <RenderValueChoice
        label={"Mode"}
        options={RENDER_FOLIAGE_MODE_OPTIONS}
        value={foliage.mode}
        onChange={(mode: RenderFoliageMode) => setFoliage({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderFoliageMode(foliage.mode)}</p>

      <RenderValueSlider
        label={"Least wind"}
        value={foliage.minSpeed}
        {...RENDER_FOLIAGE_LIMITS.minSpeed}
        isDisabled={!isEnhancedFoliage}
        format={formatPercent}
        onChange={(minSpeed: number) => setFoliage({ minSpeed })}
      />

      {FOLIAGE_MOTION.map(([key, label]) => (
        <RenderValueSlider
          key={key}
          label={label}
          value={foliage[key]}
          {...RENDER_FOLIAGE_LIMITS[key]}
          isDisabled={!isEnhancedFoliage}
          format={formatFoliageStrength}
          onChange={(value: number) => setFoliage({ [key]: value })}
        />
      ))}

      <Button size={"small"} aria-label={"Reset the foliage motion to the preset"} onClick={resetFoliage}>
        Reset to the preset
      </Button>
    </EditorPopoverToggle>
  );
}
