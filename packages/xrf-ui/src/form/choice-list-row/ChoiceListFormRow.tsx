import { filterByName } from "@xrf/search";
import { Optional } from "@xrf/types";
import { ChangeEvent, ReactElement, useCallback, useMemo, useState } from "react";

import { BaseComponentProps } from "#/element-types";
import { FormRow } from "#/form/row";
import { List, TextField } from "#/material";

import { IChoiceListFormRowOption } from "./choice-list-form-row-option";
import { IChoiceListFormRowSection, toChoiceListFormRowSections } from "./ChoiceListFormRow.utils";
import { ChoiceListFormRowItem } from "./ChoiceListFormRowItem";
import { ChoiceListFormRowSection } from "./ChoiceListFormRowSection";

/** What a filter is worth having for. Anomaly lists thirty five levels; a rifle's four submeshes do not need one. */
const DEFAULT_FILTER_FROM: number = 8;

export interface IChoiceListFormRowProps<T extends string> extends BaseComponentProps {
  label: string;
  description?: string;
  options: ReadonlyArray<IChoiceListFormRowOption<T>>;
  value: T;
  /** Above this many options the row offers a filter; below it, one would be furniture. */
  filterFrom?: number;
  isRequired?: boolean;
  isDisabled?: boolean;
  onChange: (value: T) => void;
}

/**
 * A labelled choice among many, as a list that scrolls and filters, its options under their sections' headings and
 * the one chosen described under it.
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
  const chosen: Optional<IChoiceListFormRowOption<T>> = options.find((option) => option.value === value);

  const sections: ReadonlyArray<IChoiceListFormRowSection<T>> = useMemo(() => {
    return toChoiceListFormRowSections(filterByName(options, query, (it) => it.label));
  }, [options, query]);

  const onQueryChange = useCallback((event: ChangeEvent<HTMLInputElement>) => setQuery(event.target.value), []);

  return (
    <FormRow
      label={label}
      description={description}
      fact={chosen?.description}
      controlId={id}
      isRequired={isRequired}
      isGroup
    >
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <div data-testid={dataTestId} id={controlId} className={className}>
          {isFiltered ? (
            <TextField
              fullWidth
              size={"small"}
              placeholder={"Filter by name"}
              value={query}
              disabled={isDisabled}
              slotProps={{ htmlInput: { "aria-label": `Filter ${label.toLowerCase()}` } }}
              onChange={onQueryChange}
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
            {sections.map((section: IChoiceListFormRowSection<T>) => (
              <ChoiceListFormRowSection key={section.options[0].value} label={section.label}>
                {section.options.map((option: IChoiceListFormRowOption<T>) => (
                  <ChoiceListFormRowItem
                    key={option.value}
                    option={option}
                    isSelected={option.value === value}
                    isDisabled={isDisabled}
                    onChange={onChange}
                  />
                ))}
              </ChoiceListFormRowSection>
            ))}
          </List>

          {sections.length ? null : (
            <p className={"mt-1 text-xs text-text-secondary"}>{`Nothing here matches '${query.trim()}'`}</p>
          )}
        </div>
      )}
    </FormRow>
  );
}
