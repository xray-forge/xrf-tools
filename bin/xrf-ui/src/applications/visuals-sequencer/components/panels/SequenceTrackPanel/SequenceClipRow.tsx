import { default as ArrowDownwardIcon } from "@mui/icons-material/ArrowDownward";
import { default as ArrowUpwardIcon } from "@mui/icons-material/ArrowUpward";
import { default as DeleteOutlinedIcon } from "@mui/icons-material/DeleteOutlined";
import { Box, Chip, IconButton, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  ESequenceMotionState,
  ISequenceClip,
  ISequenceMotion,
  VisualSequenceService,
} from "@/applications/visuals-sequencer/services/sequence";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { Nullable } from "@/lib/types/general";

interface ISequenceClipRowProps extends BaseComponentProps {
  clip: ISequenceClip;
  /** Where this clip sits in the track, which is what its move controls act on. */
  position: number;
  /** How long the track is, so the last clip does not offer to move further down. */
  length: number;
}

/**
 * One clip of the track: what it plays, what became of that motion, and where it sits.
 */
export function SequenceClipRow({
  "data-testid": dataTestId = "sequence-clip-row",
  id,
  className,
  clip,
  position,
  length,
}: ISequenceClipRowProps): ReactElement {
  const service: VisualSequenceService = useInjection(VisualSequenceService);

  const motion: Nullable<ISequenceMotion> = service.motions.get(clip.motion) ?? null;
  const isPlaying: boolean = service.clip?.id === clip.id;
  const frames: number = motion?.bake?.frameCount ?? 0;

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        display: "flex",
        alignItems: "center",
        gap: 0.5,
        paddingX: 1,
        paddingY: 0.5,
        borderRadius: 1,
        backgroundColor: isPlaying ? "action.selected" : "transparent",
      }}
    >
      <Typography variant={"caption"} sx={{ color: "text.disabled", flexShrink: 0, minWidth: 20 }}>
        {position + 1}
      </Typography>

      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
        <Typography
          variant={"body2"}
          sx={{ wordBreak: "break-all", cursor: "pointer" }}
          onClick={() => service.seek(position, 0)}
        >
          {clip.motion}
        </Typography>

        {motion?.state === ESequenceMotionState.READY ? (
          <Typography variant={"caption"} sx={{ color: "text.disabled" }}>
            {`${frames} frames · ${formatDuration(Math.round((motion.bake?.duration ?? 0) * 1000))}`}
          </Typography>
        ) : null}

        {motion?.state === ESequenceMotionState.BAKING ? (
          <Chip size={"small"} variant={"outlined"} label={"Baking"} />
        ) : null}

        {motion?.state === ESequenceMotionState.UNAVAILABLE ? (
          <Box>
            <Chip size={"small"} color={"error"} variant={"outlined"} label={"Unavailable"} />

            <Typography variant={"caption"} sx={{ display: "block", color: "error.main", wordBreak: "break-word" }}>
              {motion.reason}
            </Typography>
          </Box>
        ) : null}
      </Box>

      <Tooltip title={"Move earlier"}>
        <span>
          <IconButton
            size={"small"}
            aria-label={`Move ${clip.motion} earlier`}
            disabled={position === 0}
            onClick={() => service.move(clip.id, -1)}
          >
            <ArrowUpwardIcon fontSize={"small"} />
          </IconButton>
        </span>
      </Tooltip>

      <Tooltip title={"Move later"}>
        <span>
          <IconButton
            size={"small"}
            aria-label={`Move ${clip.motion} later`}
            disabled={position === length - 1}
            onClick={() => service.move(clip.id, 1)}
          >
            <ArrowDownwardIcon fontSize={"small"} />
          </IconButton>
        </span>
      </Tooltip>

      <Tooltip title={"Remove from the track"}>
        <IconButton size={"small"} aria-label={`Remove ${clip.motion}`} onClick={() => service.remove(clip.id)}>
          <DeleteOutlinedIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>
    </Box>
  );
}
