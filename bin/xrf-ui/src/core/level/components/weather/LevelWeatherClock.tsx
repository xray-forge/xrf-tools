import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { IconButton, Slider, Tooltip, Typography } from "@mui/material";
import { ReactElement, useMemo } from "react";

import { formatLevelWeatherTime, LEVEL_WEATHER_DAY } from "@/core/level/lib/weather/level-weather-time";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { IThrottledDraft, useThrottledDraft } from "@/lib/react/use-throttled-draft";

/** A minute, the slider's step. */
const MINUTE: number = 60;

interface ILevelWeatherClockProps extends BaseComponentProps {
  /** Seconds since midnight. */
  time: number;
  isPlaying: boolean;
  /** Every keyframe's time, which the slider marks. */
  keyframes?: ReadonlyArray<number>;
  isDisabled?: boolean;
  onSeek: (time: number) => void;
  onPlaying: (isPlaying: boolean) => void;
}

/**
 * The time of day: whether the clock runs, where it stands, and a slider to send it anywhere.
 */
export function LevelWeatherClock({
  "data-testid": dataTestId = "level-weather-clock",
  id,
  className,
  time,
  isPlaying,
  keyframes = [],
  isDisabled = false,
  onSeek,
  onPlaying,
}: ILevelWeatherClockProps): ReactElement {
  const draft: IThrottledDraft = useThrottledDraft(time, MINUTE, onSeek);
  const marks = useMemo(() => keyframes.map((value: number) => ({ value })), [keyframes]);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
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

        <Typography className={"font-mono tabular-nums"} variant={"body1"}>
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
    </div>
  );
}
