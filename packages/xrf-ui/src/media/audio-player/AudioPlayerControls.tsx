import { formatPlaybackTime } from "@xrf/format";
import { ReactElement } from "react";

import { PauseIcon } from "#/icons/pause";
import { PlayArrowIcon } from "#/icons/play-arrow";
import { RepeatIcon } from "#/icons/repeat";
import { VolumeUpIcon } from "#/icons/volume-up";
import { IconButton } from "#/material/icon-button";
import { Slider } from "#/material/slider";
import { Tooltip } from "#/material/tooltip";
import { Typography } from "#/material/typography";

interface IAudioPlayerControlsProps {
  isPlaying: boolean;
  isLooping: boolean;
  position: number;
  duration: number;
  volume: number;
  onTogglePlay: () => void;
  onToggleLoop: () => void;
  onVolumeChange: (value: number) => void;
}

/** Playback, looping, and volume controls for a single sound. */
export function AudioPlayerControls({
  isPlaying,
  isLooping,
  position,
  duration,
  volume,
  onTogglePlay,
  onToggleLoop,
  onVolumeChange,
}: IAudioPlayerControlsProps): ReactElement {
  return (
    <div className={"flex items-center gap-2"}>
      <Tooltip describeChild title={isPlaying ? "Pause" : "Play"}>
        <IconButton aria-label={isPlaying ? "Pause" : "Play"} color={"primary"} onClick={onTogglePlay}>
          {isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
        </IconButton>
      </Tooltip>

      <Typography className={"monospace text-text-secondary"} variant={"caption"}>
        {formatPlaybackTime(position)} / {formatPlaybackTime(duration)}
      </Typography>

      <div className={"grow"} />

      <Tooltip describeChild title={"Loop"}>
        <IconButton
          aria-label={"Loop"}
          aria-pressed={isLooping}
          color={isLooping ? "primary" : "default"}
          size={"small"}
          onClick={onToggleLoop}
        >
          <RepeatIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>

      <VolumeUpIcon className={"text-text-secondary"} fontSize={"small"} />

      <Slider
        aria-label={"Volume"}
        className={"w-24"}
        size={"small"}
        min={0}
        max={1}
        step={0.01}
        value={volume}
        onChange={(_, value) => {
          if (typeof value === "number") {
            onVolumeChange(value);
          }
        }}
      />
    </div>
  );
}
