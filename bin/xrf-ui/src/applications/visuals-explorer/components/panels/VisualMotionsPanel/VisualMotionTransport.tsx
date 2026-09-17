import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { EditorToolbarSeparator } from "@/core/shell/editor/EditorToolbarSeparator";
import { EditorViewToggle } from "@/core/shell/editor/EditorViewToggle";
import { MotionFrameSlider, MotionPlaybackRate } from "@/core/visuals/components/preview";
import { formatMotionTiming } from "@/core/visuals/lib/visual-motion";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { Nullable } from "@/lib/types/general";

/**
 * Playing what is posed: which frame, how fast, and whether it repeats.
 */
export function VisualMotionTransport({
  "data-testid": dataTestId = "visual-motion-transport",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const service: VisualMotionService = useInjection(VisualMotionService);

  const frames: number = service.frameCount;
  const duration: Nullable<number> = service.posed.value?.bake.duration ?? null;
  const speed: Nullable<number> = service.posed.value?.bake.speed ?? null;

  const onTogglePlay = useCallback(() => (service.isPlaying ? service.pause() : service.play()), [service]);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <div className={"flex items-center gap-2"}>
        <EditorIconAction
          label={service.isPlaying ? "Pause" : "Play"}
          description={service.isPlaying ? "Pause playback" : "Start playback"}
          icon={service.isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
          isDisabled={!frames}
          onClick={onTogglePlay}
        />

        <MotionFrameSlider
          ariaLabel={"Motion frame"}
          className={"mx-2"}
          frameCount={frames}
          frame={service.frame}
          onSeek={service.seek}
        />
      </div>

      <div className={"flex items-center gap-1"}>
        <Tooltip title={frames ? formatMotionTiming(frames, speed) : ""}>
          <Typography className={"grow text-text-secondary"} variant={"caption"}>
            {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
            {duration ? ` · ${formatDuration(Math.round(duration * 1000))}` : ""}
          </Typography>
        </Tooltip>

        <EditorToolbarSeparator />

        <EditorViewToggle
          label={"Loop"}
          description={service.isLooping ? "Looping" : "Play once"}
          icon={<RepeatIcon />}
          isOn={service.isLooping}
          onToggle={service.toggleLoop}
        />

        <MotionPlaybackRate fps={service.fps} onChange={service.setFps} />
      </div>

      {service.posed.error ? (
        <Typography className={"break-words text-error"} variant={"caption"}>
          {service.posed.error.message}
        </Typography>
      ) : null}
    </div>
  );
}
