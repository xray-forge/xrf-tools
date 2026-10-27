import { useInjection } from "@wirestate/react";
import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { EditorToolbarSeparator } from "@xrf/ui/editor/toolbar";
import { EditorViewToggle } from "@xrf/ui/editor/view-toggle";
import { PauseIcon } from "@xrf/ui/icons/pause";
import { PlayArrowIcon } from "@xrf/ui/icons/play-arrow";
import { RepeatIcon } from "@xrf/ui/icons/repeat";
import { SkipNextIcon } from "@xrf/ui/icons/skip-next";
import { SkipPreviousIcon } from "@xrf/ui/icons/skip-previous";
import { Paper } from "@xrf/ui/material/paper";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback } from "react";

import { MotionFrameSlider, MotionPlaybackRate } from "@/core/visuals/components/preview";
import { ISequenceClip, VisualSequenceService } from "@/tools/visuals-sequencer/services/visual-sequence";

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
    <Paper className={"flex shrink-0 items-center gap-2 bg-surface-frame p-2"} square={true} elevation={3}>
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

      <Typography className={"max-w-48.75 shrink-0"} noWrap={true} variant={"caption"} title={service.clip?.motion}>
        {service.clip ? `${service.clipIndex + 1} / ${clips.length} · ${service.clip.motion}` : "Track is empty"}
      </Typography>

      <MotionFrameSlider
        ariaLabel={"Clip frame"}
        className={"mx-2 min-w-12.5 grow basis-0"}
        frameCount={frames}
        frame={service.frame}
        onSeek={onSeek}
      />

      <Typography className={"min-w-14.75 shrink-0 text-right"} variant={"caption"}>
        {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
      </Typography>

      <EditorToolbarSeparator />

      <EditorViewToggle
        label={"Loop"}
        description={service.isLooping ? "Looping the track" : "Play the track once"}
        icon={<RepeatIcon />}
        isOn={service.isLooping}
        onToggle={service.toggleLoop}
      />

      <MotionPlaybackRate fps={service.fps} onChange={service.setFps} />
    </Paper>
  );
}
