import { Typography } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ILevelLookSectionProps extends BaseComponentProps {
  /** What the section's controls set, as its heading names it. */
  title: string;
  /** A line under the heading, saying what the controls do or why they are off. */
  note?: string;
  children: ReactNode;
}

/** One titled part of the look popover, its controls under a heading, set off from the part above by a rule. */
export function LevelLookSection({
  "data-testid": dataTestId,
  id,
  className,
  title,
  note,
  children,
}: ILevelLookSectionProps): ReactElement {
  return (
    <section
      data-testid={dataTestId}
      id={id}
      className={`flex flex-col gap-2 border-t border-divider pt-3 first:border-t-0 first:pt-0 ${className ?? ""}`}
      aria-label={title}
    >
      <Typography className={"text-text-secondary"} variant={"overline"}>
        {title}
      </Typography>

      {note ? <p className={"-mt-2 text-xs text-text-secondary"}>{note}</p> : null}

      {children}
    </section>
  );
}
