import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { InfoOutlinedIcon } from "#/icons/info-outlined";
import { cn } from "#/theme";

export interface IEditorPopoverNoteProps extends BaseComponentProps {
  /** What the note says: what a mode draws, or why the rows near it change nothing. */
  children: ReactNode;
}

/**
 * A note among a toolbar popover's rows, set apart from them as a tinted inset rather than read as one of them: what
 * the mode chosen above it draws, or why a value beside it changes nothing.
 */
export function EditorPopoverNote({
  "data-testid": dataTestId = "editor-popover-note",
  id,
  className,
  children,
}: IEditorPopoverNoteProps): ReactElement {
  return (
    <p
      data-testid={dataTestId}
      id={id}
      className={cn(
        "my-1 flex gap-1.5 rounded-control bg-action-hover px-2 py-1.5 text-xs leading-snug text-text-secondary",
        className
      )}
    >
      <InfoOutlinedIcon aria-hidden={true} className={"mt-px shrink-0 text-sm"} />
      <span className={"min-w-0"}>{children}</span>
    </p>
  );
}
