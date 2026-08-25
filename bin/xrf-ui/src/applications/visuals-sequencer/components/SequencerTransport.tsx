import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { default as SkipNextIcon } from "@mui/icons-material/SkipNext";
import { default as SkipPreviousIcon } from "@mui/icons-material/SkipPrevious";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { Box, IconButton, Paper, Popover, Slider, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { MouseEvent, ReactElement, useCallback, useState } from "react";

import { ISequenceClip, VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { LAYOUT } from "@/core/theme/tokens";
import { MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { Nullable } from "@/lib/types/general";

/**
 * Playing the track: which clip, which frame of it, and how fast.
 */
export function SequencerTransport(): ReactElement {
  const service: VisualSequenceService = useInjection(VisualSequenceService);

  const [rateAnchor, setRateAnchor] = useState<Nullable<HTMLElement>>(null);

  const clips: ReadonlyArray<ISequenceClip> = service.clips;
  const frames: number = service.frameCount;
  const isSampleRate: boolean = service.fps === MOTION_SAMPLE_FPS;

  const onTogglePlay = useCallback(() => (service.isPlaying ? service.pause() : service.play()), [service]);

  const onSeek = useCallback(
    (_: Event, value: number | Array<number>) => service.seek(service.clipIndex, value as number),
    [service]
  );

  const onStep = useCallback((offset: number) => service.seek(service.clipIndex + offset, 0), [service]);

  const onChangeFps = useCallback(
    (_: Event, value: number | Array<number>) => service.setFps(value as number),
    [service]
  );

  const onOpenRate = useCallback((event: MouseEvent<HTMLElement>) => setRateAnchor(event.currentTarget), []);

  const onCloseRate = useCallback(() => setRateAnchor(null), []);

  return (
    <Paper
      square
      elevation={3}
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 1,
        paddingX: 1,
        paddingY: 1,
        flexShrink: 0,
        backgroundColor: "background.default",
      }}
    >
      <Tooltip title={"Previous clip"}>
        <span>
          <IconButton
            size={"small"}
            aria-label={"Previous clip"}
            disabled={service.clipIndex <= 0}
            onClick={() => onStep(-1)}
          >
            <SkipPreviousIcon />
          </IconButton>
        </span>
      </Tooltip>

      <Tooltip title={service.isPlaying ? "Pause" : "Play"}>
        <span>
          <IconButton
            size={"small"}
            aria-label={service.isPlaying ? "Pause" : "Play"}
            disabled={!service.playableCount}
            onClick={onTogglePlay}
          >
            {service.isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
          </IconButton>
        </span>
      </Tooltip>

      <Tooltip title={"Next clip"}>
        <span>
          <IconButton
            size={"small"}
            aria-label={"Next clip"}
            disabled={service.clipIndex >= clips.length - 1}
            onClick={() => onStep(1)}
          >
            <SkipNextIcon />
          </IconButton>
        </span>
      </Tooltip>

      {/* Truncated rather than wrapped: with both panels open the bar is narrow, and a motion name growing a second
          line would push the controls beside it out of reach. The track panel spells the name out in full. */}
      <Typography
        noWrap
        variant={"caption"}
        title={service.clip?.motion}
        sx={{ flexShrink: 0, maxWidth: LAYOUT.motionPickerWidth * 0.75 }}
      >
        {service.clip ? `${service.clipIndex + 1} / ${clips.length} · ${service.clip.motion}` : "Track is empty"}
      </Typography>

      {/* Grown from nothing rather than from its own full width, so the slider takes what is left over instead of
          squeezing the label and the controls beside it. */}
      <Slider
        aria-label={"Clip frame"}
        size={"small"}
        min={0}
        max={Math.max(0, frames - 1)}
        value={service.frame}
        disabled={!frames}
        sx={{ marginX: 1, flexGrow: 1, flexBasis: 0, minWidth: LAYOUT.toolbarSliderWidth / 4 }}
        onChange={onSeek}
      />

      <Typography
        variant={"caption"}
        sx={{ flexShrink: 0, minWidth: LAYOUT.motionCounterWidth / 2, textAlign: "right" }}
      >
        {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
      </Typography>

      <Tooltip title={`Playback rate: ${service.fps} fps`}>
        <IconButton
          aria-label={"Playback rate"}
          aria-haspopup={"dialog"}
          color={isSampleRate ? "inherit" : "primary"}
          size={"small"}
          onClick={onOpenRate}
        >
          <SpeedIcon />
        </IconButton>
      </Tooltip>

      <Popover
        anchorEl={rateAnchor}
        open={Boolean(rateAnchor)}
        anchorOrigin={{ vertical: "top", horizontal: "center" }}
        transformOrigin={{ vertical: "bottom", horizontal: "center" }}
        onClose={onCloseRate}
      >
        <Box sx={{ paddingX: 2, paddingY: 1, width: LAYOUT.toolbarSliderWidth }}>
          <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
            Playback rate
          </Typography>

          <Slider
            size={"small"}
            min={1}
            max={120}
            value={service.fps}
            valueLabelDisplay={"auto"}
            valueLabelFormat={(value: number) => `${value} fps`}
            marks={[{ value: MOTION_SAMPLE_FPS, label: "30" }]}
            aria-label={"Frames a second"}
            onChange={onChangeFps}
          />
        </Box>
      </Popover>

      <Tooltip title={service.isLooping ? "Looping the track" : "Play the track once"}>
        <IconButton
          aria-label={"Loop"}
          size={"small"}
          sx={{ opacity: service.isLooping ? 1 : 0.45 }}
          onClick={service.toggleLoop}
        >
          <RepeatIcon />
        </IconButton>
      </Tooltip>
    </Paper>
  );
}
