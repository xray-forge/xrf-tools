import { Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { WorldWeatherReport } from "@/core/ipc/types/xrf-world";
import { LevelWeatherClock } from "@/core/level/components/weather/LevelWeatherClock";
import { ILevelWeatherControl, LEVEL_WEATHER_FACTOR_LIMITS } from "@/core/level/lib/weather/level-weather-control";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";

interface ILevelWeatherClockSectionProps extends BaseComponentProps {
  /** Seconds since midnight. */
  time: number;
  /** Every keyframe's time, which the slider marks. */
  keyframes: ReadonlyArray<number>;
  /** Where the renderer's weather stood when it last reported, or null before it has. */
  report: Nullable<WorldWeatherReport>;
  control: ILevelWeatherControl;
  /** Whether Monolith's sun table stands the sun, as it does on its engine for a game that has one. */
  isSunTabled: boolean;
  /** Whether the keyframe set by hand lights the level, standing the sun by its own angles. */
  isManual: boolean;
  onSeek: (time: number) => void;
  onPlaying: (isPlaying: boolean) => void;
  onFactor: (factor: number) => void;
  onDynamicSun: (isDynamicSun: boolean) => void;
}

/**
 * The time of day: whether the clock runs and how fast, where it stands between which keyframes, and how the sun
 * stands for it.
 */
export function LevelWeatherClockSection({
  "data-testid": dataTestId = "level-weather-clock-section",
  id,
  className,
  time,
  keyframes,
  report,
  control,
  isSunTabled,
  isManual,
  onSeek,
  onPlaying,
  onFactor,
  onDynamicSun,
}: ILevelWeatherClockSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Time"}>
      <LevelWeatherClock
        time={time}
        isPlaying={!control.isPaused}
        keyframes={keyframes}
        onSeek={onSeek}
        onPlaying={onPlaying}
      />

      {report ? (
        <EditorPanelProperty
          label={"Between"}
          value={`${formatLevelWeatherTime(report.between[0] ?? 0)}-${formatLevelWeatherTime(report.between[1] ?? 0)}, ${formatPercent(report.weight ?? 0)}`}
        />
      ) : null}

      <RenderValueSlider
        className={"mt-1"}
        label={"Speed"}
        value={Math.log10(control.factor)}
        min={Math.log10(LEVEL_WEATHER_FACTOR_LIMITS.min)}
        max={Math.log10(LEVEL_WEATHER_FACTOR_LIMITS.max)}
        step={0.01}
        format={(value: number) => `${Math.round(10 ** value)}× real time`}
        onChange={(value: number) => onFactor(10 ** value)}
      />

      {isSunTabled ? (
        <Typography className={"block text-text-secondary"} variant={"caption"}>
          {isManual
            ? "The keyframe set by hand stands the sun by its own angles."
            : "The sun stands by the game's sun table, hour by hour."}
        </Typography>
      ) : (
        <CheckboxFormRow
          label={"Dynamic sun"}
          description={
            isManual
              ? "The keyframe set by hand stands the sun by its own angles"
              : control.isDynamicSun
                ? "Computed for the time of day, as OpenXRay does by default"
                : "At the angles the keyframes write"
          }
          isChecked={control.isDynamicSun && !isManual}
          isDisabled={isManual}
          onChange={onDynamicSun}
        />
      )}
    </EditorPanelSection>
  );
}
