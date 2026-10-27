import { EditorIconAction } from "@xrf/ui/editor/icon-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AddIcon } from "@xrf/ui/icons/add";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

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
