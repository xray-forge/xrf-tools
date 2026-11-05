import { Optional } from "@xrf/types";
import { ReactElement, ReactNode, useId } from "react";

import { EditorIconAction } from "#/editor/icon-action";
import { EditorPopoverNote } from "#/editor/popover-row";
import { BaseComponentProps } from "#/element-types";
import { RestartAltIcon } from "#/icons/restart-alt";
import { Checkbox } from "#/material/checkbox";
import { Tooltip } from "#/material/tooltip";
import { Typography } from "#/material/typography";
import { cn } from "#/theme";

export interface IEditorPopoverGroupSectionProps extends BaseComponentProps {
  /** Names the section, and the toggle heading it where it has one. */
  label: string;
  /** What it does, read on hovering its name; while it cannot be turned on, why, said under it. */
  description?: string;
  /** What the mode the toggle stands at draws, said under its heading whatever it stands at. */
  note?: string;
  isOn?: boolean;
  isDisabled?: boolean;
  /** Turns the section over from a box before its name; a section with none is a titled run of rows. */
  onToggle?: () => void;
  /** Sets the section's settings back, from a button at the end of its heading; none while nothing is to set back. */
  onReset?: () => void;
  /** What the reset sets them back to, its tooltip. */
  resetLabel?: string;
  /** The section's settings, if it has any, each an `EditorPopoverRow`. */
  children?: ReactNode;
}

/**
 * One section of a toolbar popover heading its settings, as an editor's details panel heads a category: the box before
 * its name where it turns something on, its reset at the end, set apart from the section before.
 */
export function EditorPopoverGroupSection({
  "data-testid": dataTestId = "editor-popover-group-section",
  id,
  className,
  label,
  description,
  note,
  isOn = false,
  isDisabled = false,
  onToggle,
  onReset,
  resetLabel = "Reset",
  children,
}: IEditorPopoverGroupSectionProps): ReactElement {
  const controlId: string = useId();
  const labelId: string = `${controlId}-label`;
  const descriptionId: Optional<string> = description ? `${controlId}-description` : undefined;

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(
        "flex flex-col border-t border-divider pt-1.5 first-of-type:border-t-0 first-of-type:pt-0",
        className
      )}
    >
      <div className={"flex min-h-8 items-center gap-1"}>
        {onToggle ? (
          <Checkbox
            id={controlId}
            // The box's own inset, taken back so it starts at the popover's edge.
            className={"-ml-1 p-0.5"}
            size={"small"}
            checked={isOn}
            disabled={isDisabled}
            slotProps={{ input: { "aria-labelledby": labelId, "aria-describedby": descriptionId } }}
            onChange={onToggle}
          />
        ) : null}

        <Tooltip describeChild title={isDisabled ? "" : (description ?? "")} placement={"top-start"}>
          <Typography
            id={labelId}
            className={"min-w-0 flex-1 truncate"}
            component={onToggle ? "label" : "h3"}
            htmlFor={onToggle ? controlId : undefined}
            variant={"subtitle2"}
          >
            {label}
          </Typography>
        </Tooltip>

        {onReset ? (
          <EditorIconAction
            label={`${label}: ${resetLabel}`}
            description={resetLabel}
            icon={<RestartAltIcon />}
            onClick={onReset}
          />
        ) : null}
      </div>

      {description && isDisabled ? (
        <EditorPopoverNote id={descriptionId}>{description}</EditorPopoverNote>
      ) : description ? (
        <span id={descriptionId} className={"sr-only"}>
          {description}
        </span>
      ) : null}

      {note ? <EditorPopoverNote>{note}</EditorPopoverNote> : null}

      {children ? <div className={"flex flex-col pb-1"}>{children}</div> : null}
    </div>
  );
}
