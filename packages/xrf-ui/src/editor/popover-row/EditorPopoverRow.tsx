import { Optional } from "@xrf/types";
import { ReactElement, ReactNode, useId } from "react";

import { BaseComponentProps } from "#/element-types";
import { Tooltip } from "#/material/tooltip";
import { cn } from "#/theme";

/** Attributes that name a row's control by its label and describe it by its description. */
export interface IEditorPopoverRowControlProps {
  id: string;
  "aria-labelledby": string;
  "aria-describedby": Optional<string>;
}

export interface IEditorPopoverRowProps extends BaseComponentProps {
  label: string;
  /** What it does, read on hovering the label and by a screen reader, never taking a line of its own. */
  description?: string;
  /** Mutes the label beside a control that changes nothing now. */
  isDisabled?: boolean;
  /** Puts the control under the label, for one too wide to share the row. */
  isStacked?: boolean;
  /** The control, handed the attributes that name and describe it. */
  children: (props: IEditorPopoverRowControlProps) => ReactNode;
}

/**
 * One setting of a toolbar popover in a single row, as an editor's details panel lays it: the label in a column every
 * row shares, the control beside it, what it does on hovering the label.
 */
export function EditorPopoverRow({
  "data-testid": dataTestId = "editor-popover-row",
  id,
  className,
  label,
  description,
  isDisabled = false,
  isStacked = false,
  children,
}: IEditorPopoverRowProps): ReactElement {
  const controlId: string = useId();
  const labelId: string = `${controlId}-label`;
  const descriptionId: Optional<string> = description ? `${controlId}-description` : undefined;

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(isStacked ? "flex flex-col gap-1 py-1" : "flex min-h-7 items-center gap-2", className)}
    >
      <Tooltip describeChild title={description ?? ""} placement={"top-start"}>
        <label
          id={labelId}
          htmlFor={controlId}
          className={cn(
            "truncate text-xs",
            isStacked ? null : "w-popover-label shrink-0",
            isDisabled ? "text-text-disabled" : "text-text-secondary"
          )}
        >
          {label}
        </label>
      </Tooltip>

      <div className={"flex min-w-0 flex-1 items-center"}>
        {children({ id: controlId, "aria-labelledby": labelId, "aria-describedby": descriptionId })}
      </div>

      {description ? (
        <span id={descriptionId} className={"sr-only"}>
          {description}
        </span>
      ) : null}
    </div>
  );
}
