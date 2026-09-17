import { default as AddIcon } from "@mui/icons-material/Add";
import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISequenceMotionRowProps extends BaseComponentProps {
  motion: string;
  usageCount: number;
  onAdd: (motion: string) => void;
}

/** One available motion, its use in the track, and the action to add another clip. */
export function SequenceMotionRow({
  "data-testid": dataTestId = "sequence-motion-row",
  id,
  className,
  motion,
  usageCount,
  onAdd,
}: ISequenceMotionRowProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex items-center gap-2 py-0.5 pl-2 leading-panel", className)}
    >
      <Typography className={"grow break-all"} variant={"body2"}>
        {motion}
      </Typography>

      {usageCount > 0 ? (
        <Typography className={"shrink-0 text-text-disabled"} variant={"caption"}>
          {`×${usageCount}`}
        </Typography>
      ) : null}

      <EditorIconAction
        label={`Add ${motion}`}
        description={"Add to the track"}
        icon={<AddIcon />}
        onClick={() => onAdd(motion)}
      />
    </div>
  );
}
