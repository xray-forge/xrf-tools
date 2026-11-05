import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { InfoOutlinedIcon } from "#/icons/info-outlined";
import { WarningAmberIcon } from "#/icons/warning-amber";
import { cn } from "#/theme";

export interface IEditorPopoverNoteProps extends BaseComponentProps {
  /**
   * What it is: information about the rows near it, or a warning that what they hold will not do what it seems to,
   * such as a value the engine cannot reach or a request it refused. An error that stops the work is an `Alert`.
   */
  tone?: "info" | "warning";
  /** What the note says: what a mode draws, or why the rows near it change nothing. */
  children: ReactNode;
}

/**
 * A note among a toolbar popover's or panel's rows, set apart from them as a tinted inset rather than read as one of
 * them: what the mode chosen above it draws, why a value beside it changes nothing, or a warning about what it holds.
 */
export function EditorPopoverNote({
  "data-testid": dataTestId = "editor-popover-note",
  id,
  className,
  tone = "info",
  children,
}: IEditorPopoverNoteProps): ReactElement {
  const isWarning: boolean = tone === "warning";
  const Icon: typeof InfoOutlinedIcon = isWarning ? WarningAmberIcon : InfoOutlinedIcon;

  return (
    <p
      data-testid={dataTestId}
      id={id}
      className={cn(
        "my-1 flex gap-1.5 rounded-control px-2 py-1.5 text-xs leading-snug",
        isWarning ? "bg-warning/10 text-warning" : "bg-action-hover text-text-secondary",
        className
      )}
    >
      <Icon aria-hidden={true} className={"mt-px shrink-0 text-sm"} />
      <span className={"min-w-0 wrap-anywhere"}>{children}</span>
    </p>
  );
}
