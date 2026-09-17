import { default as ArrowDownwardIcon } from "@mui/icons-material/ArrowDownward";
import { default as ArrowUpwardIcon } from "@mui/icons-material/ArrowUpward";
import { default as DeleteOutlinedIcon } from "@mui/icons-material/DeleteOutlined";
import { Chip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ESequenceMotionState, ISequenceMotion } from "@/applications/visuals-sequencer/lib/sequence-motion-cache";
import { ISequenceClip, VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatSeconds } from "@/lib/format/duration";
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
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(
        "flex items-center gap-1 rounded-surface px-2 py-1",
        isPlaying ? "bg-action-current" : "bg-transparent",
        className
      )}
    >
      <Typography className={"min-w-5 shrink-0 text-text-disabled"} variant={"caption"}>
        {position + 1}
      </Typography>

      <div className={"min-w-0 grow"}>
        <Typography
          aria-label={`Seek to ${clip.motion}`}
          className={
            "block cursor-pointer border-0 bg-transparent p-0 text-left break-all text-inherit focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
          }
          component={"button"}
          type={"button"}
          variant={"body2"}
          sx={{ borderRadius: 0.5 }}
          onClick={() => service.seek(position, 0)}
        >
          {clip.motion}
        </Typography>

        {motion?.state === ESequenceMotionState.READY ? (
          <Typography className={"text-text-disabled"} variant={"caption"}>
            {`${frames} frames · ${formatSeconds(motion.bake?.duration ?? 0)}`}
          </Typography>
        ) : null}

        {motion?.state === ESequenceMotionState.BAKING ? (
          <Chip size={"small"} variant={"outlined"} label={"Baking"} />
        ) : null}

        {motion?.state === ESequenceMotionState.UNAVAILABLE ? (
          <div>
            <Chip size={"small"} color={"error"} variant={"outlined"} label={"Unavailable"} />

            <Typography className={"block break-words text-error"} variant={"caption"}>
              {motion.reason}
            </Typography>
          </div>
        ) : null}
      </div>

      <EditorIconAction
        label={`Move ${clip.motion} earlier`}
        description={position === 0 ? "Already first in the track" : "Move earlier"}
        icon={<ArrowUpwardIcon />}
        isDisabled={position === 0}
        onClick={() => service.move(clip.id, -1)}
      />

      <EditorIconAction
        label={`Move ${clip.motion} later`}
        description={position === length - 1 ? "Already last in the track" : "Move later"}
        icon={<ArrowDownwardIcon />}
        isDisabled={position === length - 1}
        onClick={() => service.move(clip.id, 1)}
      />

      <EditorIconAction
        label={`Remove ${clip.motion}`}
        description={"Remove from the track"}
        icon={<DeleteOutlinedIcon />}
        onClick={() => service.remove(clip.id)}
      />
    </div>
  );
}
