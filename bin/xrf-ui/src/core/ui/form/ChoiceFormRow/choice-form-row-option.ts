/** One value a choice offers, in display order: what a form row, Settings and the toolbar popovers list. */
export interface IChoiceFormRowOption<T extends string> {
  value: T;
  label: string;
  /** Accessible name when the visible label needs more context. */
  "aria-label"?: string;
  /** Whether it cannot be chosen, shown but greyed out. */
  isDisabled?: boolean;
}
