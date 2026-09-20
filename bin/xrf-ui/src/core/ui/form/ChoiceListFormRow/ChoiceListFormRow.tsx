import { List, ListItemButton, TextField } from "@mui/material";
import { ChangeEvent, ReactElement, useMemo, useState } from "react";

import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { FormRow } from "@/core/ui/form/FormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IChoiceListFormRowProps<T extends string> extends BaseComponentProps {
  label: string;
  description?: string;
  options: ReadonlyArray<IChoiceFormRowOption<T>>;
  value: T;
  /** Above this many options the row offers a filter; below it, one would be furniture. */
  filterFrom?: number;
  isRequired?: boolean;
  isDisabled?: boolean;
  onChange: (value: T) => void;
}

/** What a filter is worth having for. Anomaly lists thirty five levels; a rifle's four submeshes do not need one. */
const DEFAULT_FILTER_FROM: number = 8;

/**
 * A labelled choice among many, as a list that scrolls and filters.
 */
export function ChoiceListFormRow<T extends string>({
  "data-testid": dataTestId = "choice-list-form-row",
  id,
  className,
  label,
  description,
  options,
  value,
  filterFrom = DEFAULT_FILTER_FROM,
  isRequired = true,
  isDisabled = false,
  onChange,
}: IChoiceListFormRowProps<T>): ReactElement {
  const [query, setQuery] = useState<string>("");

  const isFiltered: boolean = options.length >= filterFrom;

  const shown: ReadonlyArray<IChoiceFormRowOption<T>> = useMemo(() => {
    const needle: string = query.trim().toLowerCase();

    return needle ? options.filter((it) => it.label.toLowerCase().includes(needle)) : options;
  }, [options, query]);

  return (
    <FormRow label={label} description={description} controlId={id} isRequired={isRequired} isGroup>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <div data-testid={dataTestId} id={controlId} className={className}>
          {isFiltered ? (
            <TextField
              fullWidth
              size={"small"}
              placeholder={"Filter levels"}
              value={query}
              disabled={isDisabled}
              slotProps={{ htmlInput: { "aria-label": `Filter ${label.toLowerCase()}` } }}
              onChange={(event: ChangeEvent<HTMLInputElement>) => setQuery(event.target.value)}
            />
          ) : null}

          <List
            aria-labelledby={labelId}
            aria-describedby={describedBy}
            className={"mt-1 max-h-56 overflow-y-auto rounded-surface border border-divider"}
            dense={true}
            disablePadding={true}
            role={"listbox"}
          >
            {shown.map((option: IChoiceFormRowOption<T>) => (
              <ListItemButton
                key={option.value}
                aria-selected={option.value === value}
                aria-label={option["aria-label"]}
                role={"option"}
                dense={true}
                selected={option.value === value}
                disabled={isDisabled}
                onClick={() => onChange(option.value)}
              >
                <span className={"truncate font-mono text-xs"}>{option.label}</span>
              </ListItemButton>
            ))}
          </List>

          {shown.length ? null : (
            <p className={"mt-1 text-xs text-text-secondary"}>{`Nothing here matches '${query.trim()}'`}</p>
          )}
        </div>
      )}
    </FormRow>
  );
}
