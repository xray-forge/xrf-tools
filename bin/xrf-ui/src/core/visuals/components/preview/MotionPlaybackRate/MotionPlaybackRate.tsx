import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { Box, IconButton, Popover, Slider, Tooltip, Typography } from "@mui/material";
import { MouseEvent, ReactElement, useCallback, useId, useState } from "react";

import { CONTROL, LAYOUT } from "@/core/theme/tokens";
import { MOTION_MAX_FPS, MOTION_MIN_FPS, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IMotionPlaybackRateProps extends BaseComponentProps {
  fps: number;
  onChange: (fps: number) => void;
}

/**
 * Playback-rate picker shared by single motions and sequence tracks.
 */
export function MotionPlaybackRate({
  "data-testid": dataTestId = "motion-playback-rate",
  id,
  className,
  fps,
  onChange,
}: IMotionPlaybackRateProps): ReactElement {
  const [anchor, setAnchor] = useState<Nullable<HTMLButtonElement>>(null);
  const dialogId: string = useId();
  const isOpen: boolean = anchor !== null;

  const onOpen = useCallback((event: MouseEvent<HTMLButtonElement>) => setAnchor(event.currentTarget), []);

  const onClose = useCallback(() => setAnchor(null), []);

  const onChangeFps = useCallback(
    (_: Event, value: number | Array<number>) => {
      if (typeof value === "number") {
        onChange(value);
      }
    },
    [onChange]
  );

  return (
    <>
      <Tooltip describeChild title={`Playback rate: ${fps} fps`}>
        <IconButton
          data-testid={dataTestId}
          id={id}
          className={className}
          aria-label={"Playback rate"}
          aria-description={`${fps} frames a second`}
          aria-haspopup={"dialog"}
          aria-expanded={isOpen}
          aria-controls={isOpen ? dialogId : undefined}
          color={fps === MOTION_SAMPLE_FPS ? "inherit" : "primary"}
          size={"small"}
          sx={{
            width: CONTROL.editorActionSize,
            height: CONTROL.editorActionSize,
            padding: 0,
            "& .MuiSvgIcon-root": { fontSize: CONTROL.editorActionIconSize },
          }}
          onClick={onOpen}
        >
          <SpeedIcon />
        </IconButton>
      </Tooltip>

      <Popover
        anchorEl={anchor}
        open={isOpen}
        anchorOrigin={{ vertical: "top", horizontal: "center" }}
        transformOrigin={{ vertical: "bottom", horizontal: "center" }}
        slotProps={{ paper: { id: dialogId, role: "dialog", "aria-label": "Playback rate" } }}
        onClose={onClose}
      >
        <Box sx={{ paddingX: 2, paddingY: 1, width: LAYOUT.toolbarSliderWidth }}>
          <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
            Playback rate
          </Typography>

          <Slider
            aria-label={"Frames a second"}
            size={"small"}
            min={MOTION_MIN_FPS}
            max={MOTION_MAX_FPS}
            value={fps}
            valueLabelDisplay={"auto"}
            valueLabelFormat={(value: number) => `${value} fps`}
            marks={[{ value: MOTION_SAMPLE_FPS, label: String(MOTION_SAMPLE_FPS) }]}
            onChange={onChangeFps}
          />
        </Box>
      </Popover>
    </>
  );
}
