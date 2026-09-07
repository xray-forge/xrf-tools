import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { Box, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
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
    <Box data-testid={dataTestId} id={id} className={className}>
      <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
        <EditorIconAction
          label={service.isPlaying ? "Pause" : "Play"}
          description={service.isPlaying ? "Pause playback" : "Start playback"}
          icon={service.isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
          isDisabled={!frames}
          onClick={onTogglePlay}
        />

        <MotionFrameSlider
          ariaLabel={"Motion frame"}
          frameCount={frames}
          frame={service.frame}
          sx={{ marginX: 1 }}
          onSeek={service.seek}
        />
      </Box>

      <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
        <Tooltip title={frames ? formatMotionTiming(frames, speed) : ""}>
          <Typography variant={"caption"} sx={{ flexGrow: 1, color: "text.secondary" }}>
            {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
            {duration ? ` · ${formatDuration(Math.round(duration * 1000))}` : ""}
          </Typography>
        </Tooltip>

        <MotionPlaybackRate fps={service.fps} onChange={service.setFps} />

        <EditorViewToggle
          label={"Loop"}
          description={service.isLooping ? "Looping" : "Play once"}
          icon={<RepeatIcon />}
          isOn={service.isLooping}
          onToggle={service.toggleLoop}
        />
      </Box>

      {service.posed.error ? (
        <Typography variant={"caption"} sx={{ color: "error.main", wordBreak: "break-word" }}>
          {service.posed.error.message}
        </Typography>
      ) : null}
    </Box>
  );
}
