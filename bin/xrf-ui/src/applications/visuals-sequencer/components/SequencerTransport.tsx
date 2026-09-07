import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { default as SkipNextIcon } from "@mui/icons-material/SkipNext";
import { default as SkipPreviousIcon } from "@mui/icons-material/SkipPrevious";
import { Paper, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { ISequenceClip, VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { LAYOUT } from "@/core/theme/tokens";
import { MotionFrameSlider, MotionPlaybackRate } from "@/core/visuals/components/preview";

/**
 * Playing the track: which clip, which frame of it, and how fast.
 */
export function SequencerTransport(): ReactElement {
  const service: VisualSequenceService = useInjection(VisualSequenceService);

  const clips: ReadonlyArray<ISequenceClip> = service.clips;
  const frames: number = service.frameCount;

  const onTogglePlay = useCallback(() => (service.isPlaying ? service.pause() : service.play()), [service]);

  const onSeek = useCallback((frame: number) => service.seek(service.clipIndex, frame), [service]);

  const onStep = useCallback((offset: number) => service.seek(service.clipIndex + offset, 0), [service]);

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
      <EditorIconAction
        label={"Previous clip"}
        description={"Previous clip"}
        icon={<SkipPreviousIcon />}
        isDisabled={service.clipIndex <= 0}
        onClick={() => onStep(-1)}
      />

      <EditorIconAction
        label={service.isPlaying ? "Pause" : "Play"}
        description={service.isPlaying ? "Pause playback" : "Start playback"}
        icon={service.isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
        isDisabled={!service.playableCount}
        onClick={onTogglePlay}
      />

      <EditorIconAction
        label={"Next clip"}
        description={"Next clip"}
        icon={<SkipNextIcon />}
        isDisabled={service.clipIndex >= clips.length - 1}
        onClick={() => onStep(1)}
      />

      <Typography
        noWrap
        variant={"caption"}
        title={service.clip?.motion}
        sx={{ flexShrink: 0, maxWidth: LAYOUT.motionPickerWidth * 0.75 }}
      >
        {service.clip ? `${service.clipIndex + 1} / ${clips.length} · ${service.clip.motion}` : "Track is empty"}
      </Typography>

      <MotionFrameSlider
        ariaLabel={"Clip frame"}
        frameCount={frames}
        frame={service.frame}
        sx={{ marginX: 1, flexGrow: 1, flexBasis: 0, minWidth: LAYOUT.toolbarSliderWidth / 4 }}
        onSeek={onSeek}
      />

      <Typography
        variant={"caption"}
        sx={{ flexShrink: 0, minWidth: LAYOUT.motionCounterWidth / 2, textAlign: "right" }}
      >
        {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
      </Typography>

      <MotionPlaybackRate fps={service.fps} onChange={service.setFps} />

      <EditorViewToggle
        label={"Loop"}
        description={service.isLooping ? "Looping the track" : "Play the track once"}
        icon={<RepeatIcon />}
        isOn={service.isLooping}
        onToggle={service.toggleLoop}
      />
    </Paper>
  );
}
