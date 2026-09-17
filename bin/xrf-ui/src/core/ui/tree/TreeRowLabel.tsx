import { Tooltip, Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** The one word a tree says after a name when the engine reads that file out of a volume. */
export const ARCHIVED_CAPTION: string = "db";

interface ITreeRowLabelProps extends BaseComponentProps {
  /** The row's own text, which the caption never replaces. */
  label: string;
  /** Short word said after the name, or null for a row with nothing to add. */
  caption?: Nullable<string>;
  /** What the caption means, shown on hover. */
  captionTitle?: string;
}

/**
 * A row's name, with a dim word after it where the row has something short to say.
 */
export function TreeRowLabel({ label, caption = null, captionTitle }: ITreeRowLabelProps): ReactElement {
  return caption ? (
    <div className={"flex min-w-0 items-center gap-1.5"}>
      <span className={"min-w-0 overflow-hidden text-ellipsis"}>{label}</span>

      <Tooltip title={captionTitle ?? ""}>
        <Typography
          component={"span"}
          variant={"caption"}
          sx={{ color: "text.secondary", flexShrink: 0, opacity: 0.75 }}
        >
          {caption}
        </Typography>
      </Tooltip>
    </div>
  ) : (
    (label as ReactNode as ReactElement)
  );
}
