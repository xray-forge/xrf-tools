import { useInjection } from "@wirestate/react";
import { formatDegrees, formatNumber } from "@xrf/format";
import { Optional } from "@xrf/types";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AirIcon } from "@xrf/ui/icons/air";
import { ReactElement, useCallback } from "react";

import { useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import {
  LEVEL_WEATHER_RESET_LABEL,
  useLevelWeatherReset,
} from "@/core/level/components/weather/use-level-weather-reset";
import { LEVEL_FOLIAGE_MOTION_KEYS, resetLevelFoliageKeys } from "@/core/level/lib/features/level-foliage-keys";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_WIND_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RENDER_FOLIAGE_SETTINGS } from "@/core/render/lib/features/world/render-grass-choices";
import { TRenderFoliageSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

/**
 * The wind the rain drifts on and the trees' sway, by the weather's keys; whether the trees and the grass sway at all,
 * and how the foliage moves, whose reset keeps the light through leaves the Grass popover sets.
 */
export function LevelWindAction({
  "data-testid": dataTestId = "level-wind-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { isOn, toggle } = useLevelViewToggle("isWindy");
  const { state, setNested } = useLevelFeature("grass");
  const onWeatherReset: Optional<() => void> = useLevelWeatherReset(LEVEL_MANUAL_WEATHER_WIND_KEYS);
  const onMotionReset: Optional<() => void> = useLevelFeatureReset(
    "grass",
    resetLevelFoliageKeys(features.grass, settings.grass.foliage, LEVEL_FOLIAGE_MOTION_KEYS)
  );

  const manual: ILevelManualWeather = weatherService.shown;
  const foliage: TRenderFoliageSettings = state.value.foliage;

  const setFoliage = useCallback(
    (part: Partial<TRenderFoliageSettings>): void => setNested("foliage", part),
    [setNested]
  );

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
      resetLabel={LEVEL_WEATHER_RESET_LABEL}
      onToggle={toggle}
      onReset={onWeatherReset}
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

      <EditorPopoverGroupSection
        label={"Foliage motion"}
        resetLabel={LEVEL_SETTINGS_RESET_LABEL}
        onReset={onMotionReset}
      >
        <RenderSettingEntries
          settings={RENDER_FOLIAGE_SETTINGS}
          names={LEVEL_FOLIAGE_MOTION_KEYS}
          group={foliage}
          features={drawn}
          onSet={setFoliage}
        />
      </EditorPopoverGroupSection>
    </EditorPopoverToggle>
  );
}
