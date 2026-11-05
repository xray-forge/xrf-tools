import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { Typography } from "#/material/typography";
import { cn } from "#/theme";

export interface IEditorPopoverContentProps extends BaseComponentProps {
  /** The popover's name, its heading. */
  label: string;
  /** Whether it takes the wider width, for rows of several named choices. */
  isWide?: boolean;
  children: ReactNode;
}

/**
 * What a toolbar popover holds, under its name: every popover takes one width, so its rows line up with the next one's.
 */
export function EditorPopoverContent({
  "data-testid": dataTestId = "editor-popover-content",
  id,
  className,
  label,
  isWide = false,
  children,
}: IEditorPopoverContentProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex flex-col gap-1 px-3 pt-1 pb-3", isWide ? "w-popover-wide" : "w-popover", className)}
    >
      <Typography className={"text-text-secondary"} variant={"overline"}>
        {label}
      </Typography>

      {children}
    </div>
  );
}
