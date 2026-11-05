import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { Select, SelectChangeEvent } from "#/material/select";
import { CONTROL } from "#/theme";
import { POPOVER_INPUT_SX } from "#/theme/sx";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

export interface IEditorPopoverSelectRowProps<T extends string> extends BaseComponentProps {
  label: string;
  /** What it picks, read on hovering the label and by a screen reader. */
  description?: string;
  value: T;
  /** What the closed select reads for a value, such as None for an empty one; the value itself by default. */
  renderValue?: (value: T) => ReactNode;
  isDisabled?: boolean;
  /** The options, each a `MenuItem` whose value is one of `T`. */
  children: ReactNode;
  onChange: (value: T) => void;
}

/**
 * One of a list of named values in a single row of a toolbar popover: the label in the column every row shares, the
 * select beside it, its menu holding however much each option says.
 */
export function EditorPopoverSelectRow<T extends string>({
  "data-testid": dataTestId = "editor-popover-select-row",
  id,
  className,
  label,
  description,
  value,
  renderValue,
  isDisabled = false,
  children,
  onChange,
}: IEditorPopoverSelectRowProps<T>): ReactElement {
  return (
    <EditorPopoverRow id={id} className={className} label={label} description={description} isDisabled={isDisabled}>
      {({
        id: controlId,
        "aria-labelledby": labelId,
        "aria-describedby": descriptionId,
      }: IEditorPopoverRowControlProps) => (
        <Select<T>
          data-testid={dataTestId}
          sx={POPOVER_INPUT_SX}
          size={"small"}
          fullWidth
          displayEmpty
          value={value}
          disabled={isDisabled}
          labelId={labelId}
          SelectDisplayProps={{ id: controlId, "aria-describedby": descriptionId }}
          MenuProps={{ slotProps: { paper: { sx: { maxHeight: CONTROL.selectMenuMaxHeight } } } }}
          renderValue={renderValue}
          onChange={(event: SelectChangeEvent<T>) => onChange(event.target.value as T)}
        >
          {children}
        </Select>
      )}
    </EditorPopoverRow>
  );
}
