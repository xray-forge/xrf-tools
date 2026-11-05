import { ReactElement, ReactNode } from "react";

import { EditorIconAction } from "#/editor/icon-action";
import { BaseComponentProps } from "#/element-types";
import { RestartAltIcon } from "#/icons/restart-alt";
import { Typography } from "#/material/typography";
import { cn } from "#/theme";

export interface IEditorPopoverContentProps extends BaseComponentProps {
  /** The popover's name, its heading. */
  label: string;
  /** Whether it takes the wider width, for rows of several named choices. */
  isWide?: boolean;
  /**
   * Sets back what every section of the popover sets, from a button at the end of its heading, where one reset covers
   * them all; none while nothing is to set back.
   */
  onReset?: () => void;
  /** What the reset sets them back to, its tooltip. */
  resetLabel?: string;
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
  onReset,
  resetLabel = "Reset",
  children,
}: IEditorPopoverContentProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex flex-col gap-1 px-3 pt-1 pb-3", isWide ? "w-popover-wide" : "w-popover", className)}
    >
      <div className={"flex min-h-8 items-center justify-between gap-1"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          {label}
        </Typography>

        {onReset ? (
          <EditorIconAction
            label={`${label}: ${resetLabel}`}
            description={resetLabel}
            icon={<RestartAltIcon />}
            onClick={onReset}
          />
        ) : null}
      </div>

      {children}
    </div>
  );
}
