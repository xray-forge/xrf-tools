import { formatPercent } from "@xrf/format";
import { WorldWeatherReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { EditorPopoverRow, IEditorPopoverRowControlProps } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { LevelWeatherClock } from "@/core/level/components/weather/LevelWeatherClock";
import { ILevelWeatherControl, LEVEL_WEATHER_FACTOR_LIMITS } from "@/core/level/lib/weather/level-weather-control";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

interface ILevelWeatherTimeProps extends BaseComponentProps {
  /** Seconds since midnight. */
  time: number;
  /** Every keyframe's time, which the clock marks. */
  keyframes: ReadonlyArray<number>;
  /** Where the renderer's weather stood when it last reported, or null before it has or while lit by hand. */
  report: Nullable<WorldWeatherReport>;
  control: ILevelWeatherControl;
  onSeek: (time: number) => void;
  onPlaying: (isPlaying: boolean) => void;
  onFactor: (factor: number) => void;
}

/**
 * The time of day and how it runs, a row each: the clock with the cycle's keyframes marked, the two keyframes it stands
 * between and how far, and how many times real time it runs. The Weather panel and the toolbar's Weather popover show
 * the same.
 */
export function LevelWeatherTime({
  "data-testid": dataTestId = "level-weather-time",
  id,
  className,
  time,
  keyframes,
  report,
  control,
  onSeek,
  onPlaying,
  onFactor,
}: ILevelWeatherTimeProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <LevelWeatherClock
        time={time}
        isPlaying={!control.isPaused}
        keyframes={keyframes}
        onSeek={onSeek}
        onPlaying={onPlaying}
      />

      {report ? (
        <EditorPopoverRow
          label={"Between"}
          description={"The two keyframes the time stands between, and how far it is toward the second."}
        >
          {({ id: valueId }: IEditorPopoverRowControlProps) => (
            <span id={valueId} className={"text-xs tabular-nums"}>
              {`${formatLevelWeatherTime(report.between[0] ?? 0)}-${formatLevelWeatherTime(report.between[1] ?? 0)}, ${formatPercent(report.weight ?? 0)}`}
            </span>
          )}
        </EditorPopoverRow>
      ) : null}

      <RenderValueSlider
        label={"Speed"}
        description={"How many times faster than real time the clock runs."}
        value={Math.log10(control.factor)}
        min={Math.log10(LEVEL_WEATHER_FACTOR_LIMITS.min)}
        max={Math.log10(LEVEL_WEATHER_FACTOR_LIMITS.max)}
        step={0.01}
        format={(value: number) => `${Math.round(10 ** value)}×`}
        onChange={(value: number) => onFactor(10 ** value)}
      />
    </div>
  );
}
