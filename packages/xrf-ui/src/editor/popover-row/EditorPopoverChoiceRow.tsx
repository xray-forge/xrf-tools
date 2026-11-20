import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ToggleButton, ToggleButtonGroup } from "#/button";
import { BaseComponentProps } from "#/element-types";
import { IChoiceFormRowOption } from "#/form/choice-row";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

export interface IEditorPopoverChoiceRowProps<T extends string> extends BaseComponentProps {
  label: string;
  /** What it does, read on hovering its label. */
  description?: string;
  options: ReadonlyArray<IChoiceFormRowOption<T>>;
  value: T;
  /** Whether the values stack one under another, under the label, for names too long to share a row. */
  isStacked?: boolean;
  isDisabled?: boolean;
  onChange: (value: T) => void;
}

/**
 * One of a few values in a single row of a toolbar popover: the label in the column every row shares and the values
 * beside it, one kept selected.
 */
export function EditorPopoverChoiceRow<T extends string>({
  "data-testid": dataTestId = "editor-popover-choice-row",
  id,
  className,
  label,
  description,
  options,
  value,
  isStacked = false,
  isDisabled = false,
  onChange,
}: IEditorPopoverChoiceRowProps<T>): ReactElement {
  return (
    <EditorPopoverRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      isDisabled={isDisabled}
      isStacked={isStacked}
    >
      {(control: IEditorPopoverRowControlProps) => (
        <ToggleButtonGroup
          {...control}
          exclusive={true}
          fullWidth={true}
          orientation={isStacked ? "vertical" : "horizontal"}
          color={"primary"}
          size={"small"}
          value={value}
          onChange={(_, next: Nullable<T>) => {
            if (next !== null) {
              onChange(next);
            }
          }}
        >
          {options.map((option: IChoiceFormRowOption<T>) => (
            <ToggleButton
              key={option.value}
              aria-label={option["aria-label"]}
              className={"py-0.5 text-xs"}
              value={option.value}
              disabled={isDisabled || option.isDisabled}
            >
              {option.label}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      )}
    </EditorPopoverRow>
  );
}
