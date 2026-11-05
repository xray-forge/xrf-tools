import { Optional } from "@xrf/types";
import { ReactElement, useId } from "react";

import { BaseComponentProps } from "#/element-types";
import { Checkbox } from "#/material/checkbox";
import { Tooltip } from "#/material/tooltip";
import { cn } from "#/theme";

export interface IEditorPopoverSwitchRowProps extends BaseComponentProps {
  label: string;
  /** What it switches, read on hovering the label and by a screen reader. */
  description?: string;
  isChecked: boolean;
  isDisabled?: boolean;
  /** Sets the row in under the one before, as a switch that only matters while that one is on. */
  isNested?: boolean;
  onChange: (isChecked: boolean) => void;
}

/**
 * One of a list of switches with no value but on or off, as an editor's show menu lists them: the box before its name,
 * the name taking the row.
 */
export function EditorPopoverSwitchRow({
  "data-testid": dataTestId = "editor-popover-switch-row",
  id,
  className,
  label,
  description,
  isChecked,
  isDisabled = false,
  isNested = false,
  onChange,
}: IEditorPopoverSwitchRowProps): ReactElement {
  const controlId: string = useId();
  const labelId: string = `${controlId}-label`;
  const descriptionId: Optional<string> = description ? `${controlId}-description` : undefined;

  return (
    <div id={id} className={cn("flex min-h-7 items-center gap-1", isNested ? "pl-6" : null, className)}>
      <Checkbox
        data-testid={dataTestId}
        id={controlId}
        // The box's own inset, taken back so it starts at the popover's edge.
        className={"-ml-1 p-0.5"}
        size={"small"}
        checked={isChecked}
        disabled={isDisabled}
        slotProps={{ input: { "aria-labelledby": labelId, "aria-describedby": descriptionId } }}
        onChange={(_, checked: boolean) => onChange(checked)}
      />

      <Tooltip describeChild title={description ?? ""} placement={"top-start"}>
        <label
          id={labelId}
          htmlFor={controlId}
          className={cn("min-w-0 flex-1 truncate text-sm", isDisabled ? "text-text-disabled" : null)}
        >
          {label}
        </label>
      </Tooltip>

      {description ? (
        <span id={descriptionId} className={"sr-only"}>
          {description}
        </span>
      ) : null}
    </div>
  );
}
