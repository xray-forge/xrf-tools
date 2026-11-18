import { formatPlaybackTime } from "@xrf/format";
import { IThrottledDraft, useThrottledDraft } from "@xrf/react/throttled-draft";
import { ReactElement } from "react";

import { IconButton } from "#/button";
import { Slider } from "#/form/control";
import { PauseIcon } from "#/icons/pause";
import { PlayArrowIcon } from "#/icons/play-arrow";
import { RepeatIcon } from "#/icons/repeat";
import { VolumeUpIcon } from "#/icons/volume-up";
import { Tooltip } from "#/overlay";
import { Typography } from "#/typography";

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

/** The volume slider's step. */
const VOLUME_STEP: number = 0.01;

/**
 * Playback, looping, and volume controls for a single sound. The volume follows the hand at once and tells its owner at
 * most once an interval while dragged, as the kit's other sliders do, since an owner may keep it on disk.
 */
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
  const volumeDraft: IThrottledDraft = useThrottledDraft(volume, VOLUME_STEP, onVolumeChange);

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
        step={VOLUME_STEP}
        value={volumeDraft.value}
        onChange={volumeDraft.onChange}
        onChangeCommitted={volumeDraft.onChangeCommitted}
      />
    </div>
  );
}
