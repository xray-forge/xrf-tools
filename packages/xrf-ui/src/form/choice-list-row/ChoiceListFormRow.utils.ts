import { Optional } from "@xrf/types";

import { IChoiceListFormRowOption } from "./choice-list-form-row-option";

/** A run of neighbouring options under one heading, or under none. */
export interface IChoiceListFormRowSection<T extends string> {
  label: Optional<string>;
  options: Array<IChoiceListFormRowOption<T>>;
}

/**
 * @param options - The options shown, in the order offered.
 * @returns Them in runs of neighbours sharing a section, a section recurring apart from its run starting another.
 */
export function toChoiceListFormRowSections<T extends string>(
  options: ReadonlyArray<IChoiceListFormRowOption<T>>
): ReadonlyArray<IChoiceListFormRowSection<T>> {
  const sections: Array<IChoiceListFormRowSection<T>> = [];

  for (const option of options) {
    const last: Optional<IChoiceListFormRowSection<T>> = sections.at(-1);

    if (last && last.label === option.section) {
      last.options.push(option);
    } else {
      sections.push({ label: option.section, options: [option] });
    }
  }

  return sections;
}
