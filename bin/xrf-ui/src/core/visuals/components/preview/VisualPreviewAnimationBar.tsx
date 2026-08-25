import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { Autocomplete, Box, IconButton, Paper, Popover, Slider, TextField, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { MouseEvent, ReactElement, SyntheticEvent, useCallback, useEffect, useState } from "react";

import { LAYOUT } from "@/core/theme/tokens";
import { MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { formatDuration } from "@/lib/format/duration";
import { Nullable } from "@/lib/types/general";

export interface IVisualPreviewAnimationBarProps {
  /**
   * The model whose motions these are.
   *
   * Taken as a prop rather than read from a service so listing follows what is on screen: the backend answers
   * `list_motions` about whatever it currently has open, and a model still loading is not that yet.
   */
  model: Nullable<IVisualModelViews>;
}

/**
 * Picking and playing one of the open visual's motions.
 *
 * An autocomplete rather than a plain select because a character references thousands of motions - a measured actor
 * offers 2,500 - so the list has to be typed into rather than scrolled. Dragging the slider pauses, because otherwise
 * playback and the drag fight over the same frame; picking another motion does not, so two of them can be compared at
 * the same speed.
 */
export function VisualPreviewAnimationBar({ model }: IVisualPreviewAnimationBarProps): ReactElement {
  const service: VisualMotionService = useInjection(VisualMotionService);

  const [rateAnchor, setRateAnchor] = useState<Nullable<HTMLElement>>(null);

  const names: Array<string> = service.motions.value ?? [];
  const posed: Nullable<string> = service.posed.value?.bake.name ?? null;
  const frames: number = service.frameCount;
  const duration: Nullable<number> = service.posed.value?.bake.duration ?? null;
  const isSampleRate: boolean = service.fps === MOTION_SAMPLE_FPS;

  const onPick = useCallback(
    (_: SyntheticEvent, name: Nullable<string>) => {
      if (name) {
        void service.open(name);
      }
    },
    [service]
  );

  const onSeek = useCallback((_: Event, value: number | Array<number>) => service.seek(value as number), [service]);

  const onTogglePlay = useCallback(() => (service.isPlaying ? service.pause() : service.play()), [service]);

  const onChangeFps = useCallback(
    (_: Event, value: number | Array<number>) => service.setFps(value as number),
    [service]
  );

  const onOpenRate = useCallback((event: MouseEvent<HTMLElement>) => setRateAnchor(event.currentTarget), []);

  const onCloseRate = useCallback(() => setRateAnchor(null), []);

  // Listed when a model lands rather than when it is asked for: naming motions means reading every animation file the
  // visual references, and the bar only exists for a visual that has some. Depending on the model is what brings the
  // list back for the next one, whose motions are a different list even though this bar never unmounted.
  useEffect(() => void service.list(), [service, model]);

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
      <Autocomplete
        size={"small"}
        options={names}
        value={posed}
        loading={service.motions.isLoading}
        disabled={!names.length && !service.motions.isLoading}
        sx={{
          width: LAYOUT.motionPickerWidth,
          flexShrink: 0,
          "& .MuiInputBase-root, & .MuiAutocomplete-input": { cursor: "pointer" },
        }}
        renderInput={(parameters) => (
          <TextField
            {...parameters}
            placeholder={service.motions.isLoading ? "Listing motions…" : "Pick a motion"}
            // Merged rather than replaced: the autocomplete's own `input` slot carries the ref its popup anchors to.
            slotProps={{
              ...parameters.slotProps,
              htmlInput: { ...parameters.slotProps.htmlInput, "aria-label": "Motion" },
            }}
          />
        )}
        onChange={onPick}
      />

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

      <Typography variant={"caption"} sx={{ minWidth: LAYOUT.motionCounterWidth, textAlign: "right" }}>
        {frames ? `${service.frame + 1} / ${frames}` : "0 / 0"}
        {duration ? ` · ${formatDuration(Math.round(duration * 1000))}` : ""}
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
            // The rate the format samples at, marked so the honest speed is one click away from any other.
            marks={[{ value: MOTION_SAMPLE_FPS, label: "30" }]}
            aria-label={"Frames a second"}
            onChange={onChangeFps}
          />
        </Box>
      </Popover>

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

      {service.posed.error ? (
        <Typography variant={"caption"} sx={{ color: "error.main", wordBreak: "break-word" }}>
          {service.posed.error.message}
        </Typography>
      ) : null}
    </Paper>
  );
}
