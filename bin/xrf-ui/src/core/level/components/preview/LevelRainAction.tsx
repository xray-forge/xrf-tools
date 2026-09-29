import { default as WaterDropIcon } from "@mui/icons-material/WaterDrop";
import { ReactElement } from "react";

import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { LevelWeatherCollectionSelect } from "@/core/level/components/weather/LevelWeatherCollectionSelect";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherTriple } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatSeconds } from "@/lib/format/duration";
import { formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const RAIN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "rainDensity",
  "rainColor",
  "thunderboltCollection",
  "thunderboltPeriod",
  "thunderboltDuration",
];

interface ILevelRainActionProps extends BaseComponentProps {
  options: ILevelViewOptions;
  /** The keyframe on screen: the one set by hand, or the weather's mix, which the first edit seeds it from. */
  manual: ILevelManualWeather;
  /** Every thunderbolt collection of the game. */
  collections: ReadonlyArray<ThunderboltCollection>;
  onToggle: (option: keyof ILevelViewOptions) => void;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * How hard it rains and the streaks' colour, and the bolts that strike with it; whether either happens at all.
 */
export function LevelRainAction({
  "data-testid": dataTestId = "level-rain-action",
  id,
  className,
  options,
  manual,
  collections,
  onToggle,
  onEdit,
}: ILevelRainActionProps): ReactElement {
  const { isRainy, isThundering } = options;
  const isRaining: boolean = isRainy && manual.rainDensity > 0;
  const isStriking: boolean = isThundering && manual.thunderboltCollection !== "";

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Rain"}
      description={[
        isRaining ? `Rain at ${formatPercent(manual.rainDensity)}` : isRainy ? "No rain" : "Rain off",
        isStriking ? `bolts of ${manual.thunderboltCollection}` : isThundering ? "no thunder" : "thunder off",
      ].join(", ")}
      icon={<WaterDropIcon />}
      isActive={isRaining || isStriking}
    >
      <EditorPopoverGroupSection label={"Rain"} isOn={isRainy} onToggle={() => onToggle("isRainy")}>
        <RenderValueSlider
          label={"rain_density"}
          value={manual.rainDensity}
          {...LEVEL_MANUAL_WEATHER_LIMITS.rainDensity}
          format={formatPercent}
          onChange={(rainDensity: number) => onEdit({ rainDensity })}
        />

        <LevelWeatherVectorField
          label={"rain_color"}
          isColor
          value={manual.rainColor}
          onChange={(rainColor) => onEdit({ rainColor: toLevelWeatherTriple(rainColor) })}
        />
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection label={"Thunder"} isOn={isThundering} onToggle={() => onToggle("isThundering")}>
        <LevelWeatherCollectionSelect
          value={manual.thunderboltCollection}
          collections={collections}
          onChange={(thunderboltCollection: string) => onEdit({ thunderboltCollection })}
        />

        <RenderValueSlider
          label={"thunderbolt_period"}
          value={manual.thunderboltPeriod}
          {...LEVEL_MANUAL_WEATHER_LIMITS.thunderboltPeriod}
          format={formatSeconds}
          onChange={(thunderboltPeriod: number) => onEdit({ thunderboltPeriod })}
        />

        <RenderValueSlider
          label={"thunderbolt_duration"}
          value={manual.thunderboltDuration}
          {...LEVEL_MANUAL_WEATHER_LIMITS.thunderboltDuration}
          format={formatSeconds}
          onChange={(thunderboltDuration: number) => onEdit({ thunderboltDuration })}
        />
      </EditorPopoverGroupSection>

      <LevelWeatherResetButton keys={RAIN_KEYS} onEdit={onEdit} />
    </EditorPopoverGroup>
  );
}
