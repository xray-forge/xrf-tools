import { IRendererWeatherReport } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelWeatherClock } from "@/core/level/components/weather/LevelWeatherClock";
import { ILevelWeatherControl } from "@/core/level/lib/weather/level-weather-control";
import { LEVEL_WEATHER_FACTOR_LIMITS } from "@/core/level/lib/weather/level-weather-memory";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";

interface ILevelWeatherClockSectionProps extends BaseComponentProps {
  /** Seconds since midnight. */
  time: number;
  /** Every keyframe's time, which the slider marks. */
  keyframes: ReadonlyArray<number>;
  /** Where the renderer's weather stood when it last reported, or null before it has. */
  report: Nullable<IRendererWeatherReport>;
  control: ILevelWeatherControl;
  isDisabled?: boolean;
  onSeek: (time: number) => void;
  onPlaying: (isPlaying: boolean) => void;
  onFactor: (factor: number) => void;
}

/**
 * The time of day: where the clock stands between which keyframes, whether it runs, and how fast.
 */
export function LevelWeatherClockSection({
  "data-testid": dataTestId = "level-weather-clock-section",
  id,
  className,
  time,
  keyframes,
  report,
  control,
  isDisabled = false,
  onSeek,
  onPlaying,
  onFactor,
}: ILevelWeatherClockSectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Time"}>
      <LevelWeatherClock
        time={time}
        isPlaying={!control.isPaused}
        keyframes={keyframes}
        isDisabled={isDisabled}
        onSeek={onSeek}
        onPlaying={onPlaying}
      />

      {report ? (
        <EditorPanelProperty
          label={"Between"}
          value={`${formatLevelWeatherTime(report.between[0])} and ${formatLevelWeatherTime(
            report.between[1]
          )}, ${formatPercent(report.weight)} of the way`}
        />
      ) : null}

      <RenderValueSlider
        className={"mt-2"}
        label={"Speed"}
        value={Math.log10(control.factor)}
        min={Math.log10(LEVEL_WEATHER_FACTOR_LIMITS.min)}
        max={Math.log10(LEVEL_WEATHER_FACTOR_LIMITS.max)}
        step={0.01}
        format={(value: number) => `${Math.round(10 ** value)}× real time`}
        onChange={(value: number) => onFactor(10 ** value)}
      />
    </EditorPanelSection>
  );
}
