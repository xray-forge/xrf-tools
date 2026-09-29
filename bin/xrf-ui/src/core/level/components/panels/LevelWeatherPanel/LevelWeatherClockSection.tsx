import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { IconButton, Slider, Tooltip, Typography } from "@mui/material";
import { IRendererWeatherReport } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { ILevelWeatherControl } from "@/core/level/lib/weather/level-weather-control";
import { LEVEL_WEATHER_FACTOR_LIMITS } from "@/core/level/lib/weather/level-weather-memory";
import { formatLevelWeatherTime, LEVEL_WEATHER_DAY } from "@/core/level/lib/weather/level-weather-time";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatPercent } from "@/lib/format/number";
import { IThrottledDraft, useThrottledDraft } from "@/lib/react/use-throttled-draft";

/** A minute, the time slider's step. */
const MINUTE: number = 60;

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
  const draft: IThrottledDraft = useThrottledDraft(time, MINUTE, onSeek);
  const marks = useMemo(() => keyframes.map((value: number) => ({ value })), [keyframes]);
  const isPlaying: boolean = !control.isPaused;

  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Time"}>
      <div className={"flex items-center gap-2"}>
        <Tooltip title={isPlaying ? "Pause the clock" : "Run the clock"}>
          <span>
            <IconButton
              size={"small"}
              disabled={isDisabled}
              aria-label={isPlaying ? "Pause" : "Play"}
              onClick={() => onPlaying(!isPlaying)}
            >
              {isPlaying ? <PauseIcon fontSize={"small"} /> : <PlayArrowIcon fontSize={"small"} />}
            </IconButton>
          </span>
        </Tooltip>

        <Typography className={"font-mono tabular-nums"} variant={"h6"}>
          {formatLevelWeatherTime(draft.value, true)}
        </Typography>
      </div>

      <Slider
        size={"small"}
        min={0}
        max={LEVEL_WEATHER_DAY - MINUTE}
        step={MINUTE}
        marks={marks}
        value={draft.value}
        disabled={isDisabled}
        aria-label={"Time of day"}
        getAriaValueText={(value: number) => formatLevelWeatherTime(value)}
        onChange={draft.onChange}
        onChangeCommitted={draft.onChangeCommitted}
      />

      {report?.keyframes && keyframes.length ? (
        <EditorPanelProperty
          label={"Between"}
          value={`${formatLevelWeatherTime(keyframes[report.keyframes[0]] ?? 0)} and ${formatLevelWeatherTime(
            keyframes[report.keyframes[1]] ?? 0
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
