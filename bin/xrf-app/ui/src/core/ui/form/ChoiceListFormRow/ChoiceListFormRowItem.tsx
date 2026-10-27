import { ListItemButton, Tooltip } from "@xrf/ui/material";
import { ReactElement, useCallback } from "react";

import { IChoiceListFormRowOption } from "./choice-list-form-row-option";

interface IChoiceListFormRowItemProps<T extends string> {
  option: IChoiceListFormRowOption<T>;
  isSelected: boolean;
  isDisabled: boolean;
  onChange: (value: T) => void;
}

/**
 * One option of a choice list, its description as its tooltip.
 */
export function ChoiceListFormRowItem<T extends string>({
  option,
  isSelected,
  isDisabled,
  onChange,
}: IChoiceListFormRowItemProps<T>): ReactElement {
  const onSelect = useCallback(() => onChange(option.value), [onChange, option.value]);

  return (
    <Tooltip describeChild disableInteractive placement={"right"} title={isDisabled ? "" : (option.description ?? "")}>
      <ListItemButton
        aria-selected={isSelected}
        aria-label={option["aria-label"]}
        role={"option"}
        dense={true}
        selected={isSelected}
        disabled={isDisabled}
        onClick={onSelect}
      >
        <span className={"truncate font-mono text-xs"}>{option.label}</span>
      </ListItemButton>
    </Tooltip>
  );
}
