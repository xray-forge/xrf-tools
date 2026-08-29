import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { Box, IconButton, Popover, Slider, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { MouseEvent, ReactElement, useCallback, useState } from "react";

import { LAYOUT } from "@/core/theme/tokens";
import { formatMotionTiming, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { Nullable } from "@/lib/types/general";

export interface IVisualMotionTransportProps extends BaseComponentProps {}

/**
 * Playing what is posed: which frame, how fast, and whether it repeats.
 */
export function VisualMotionTransport({
  "data-testid": dataTestId = "visual-motion-transport",
  id,
  className,
}: IVisualMotionTransportProps = {}): ReactElement {
  const service: VisualMotionService = useInjection(VisualMotionService);

  const [rateAnchor, setRateAnchor] = useState<Nullable<HTMLElement>>(null);

  const frames: number = service.frameCount;
  const duration: Nullable<number> = service.posed.value?.bake.duration ?? null;
  const speed: Nullable<number> = service.posed.value?.bake.speed ?? null;
  const isSampleRate: boolean = service.fps === MOTION_SAMPLE_FPS;

  const onSeek = useCallback((_: Event, value: number | Array<number>) => service.seek(value as number), [service]);

  const onTogglePlay = useCallback(() => (service.isPlaying ? service.pause() : service.play()), [service]);

  const onChangeFps = useCallback(
    (_: Event, value: number | Array<number>) => service.setFps(value as number),
    [service]
  );

  const onOpenRate = useCallback((event: MouseEvent<HTMLElement>) => setRateAnchor(event.currentTarget), []);

  const onCloseRate = useCallback(() => setRateAnchor(null), []);

  return (
    <Box data-testid={dataTestId} id={id} className={className}>
      <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
        <Tooltip title={service.isPlaying ? "Pause" : "Play"}>
          <span>
            <IconButton
              size={"small"}
              aria-label={service.isPlaying ? "Pause" : "Play"}
              disabled={!frames}
              onClick={onTogglePlay}
            >
              {service.isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
            </IconButton>
          </span>
        </Tooltip>

        <Slider
          aria-label={"Motion frame"}
          size={"small"}
          min={0}
          max={Math.max(0, frames - 1)}
          value={service.frame}
          disabled={!frames}
          sx={{ marginX: 1 }}
          onChange={onSeek}
        />
      </Box>

      <Box sx={{ display: "flex", alignItems: "center", gap: 0.5 }}>
        <Tooltip title={frames ? formatMotionTiming(frames, speed) : ""}>
          <Typography variant={"caption"} sx={{ flexGrow: 1, color: "text.secondary" }}>
            {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
            {duration ? ` · ${formatDuration(Math.round(duration * 1000))}` : ""}
          </Typography>
        </Tooltip>

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

        <Tooltip title={service.isLooping ? "Looping" : "Play once"}>
          <IconButton
            aria-label={"Loop"}
            size={"small"}
            sx={{ opacity: service.isLooping ? 1 : 0.45 }}
            onClick={service.toggleLoop}
          >
            <RepeatIcon />
          </IconButton>
        </Tooltip>
      </Box>

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
            // The rate the format samples at, marked so the honest speed is one click away from any other.
            marks={[{ value: MOTION_SAMPLE_FPS, label: "30" }]}
            aria-label={"Frames a second"}
            onChange={onChangeFps}
          />
        </Box>
      </Popover>

      {service.posed.error ? (
        <Typography variant={"caption"} sx={{ color: "error.main", wordBreak: "break-word" }}>
          {service.posed.error.message}
        </Typography>
      ) : null}
    </Box>
  );
}
