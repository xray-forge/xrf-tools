import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { default as VolumeUpIcon } from "@mui/icons-material/VolumeUp";
import { Box, IconButton, Slider, Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

import { formatPlaybackTime } from "@/lib/media/waveform";

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
    <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
      <Tooltip describeChild title={isPlaying ? "Pause" : "Play"}>
        <IconButton aria-label={isPlaying ? "Pause" : "Play"} color={"primary"} onClick={onTogglePlay}>
          {isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
        </IconButton>
      </Tooltip>

      <Typography variant={"caption"} className={"monospace"} sx={{ color: "text.secondary" }}>
        {formatPlaybackTime(position)} / {formatPlaybackTime(duration)}
      </Typography>

      <Box sx={{ flexGrow: 1 }} />

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

      <VolumeUpIcon fontSize={"small"} sx={{ color: "text.secondary" }} />

      <Slider
        aria-label={"Volume"}
        size={"small"}
        min={0}
        max={1}
        step={0.01}
        value={volume}
        sx={{ width: 96 }}
        onChange={(_, value) => {
          if (typeof value === "number") {
            onVolumeChange(value);
          }
        }}
      />
    </Box>
  );
}
