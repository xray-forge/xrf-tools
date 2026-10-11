/** One value a choice offers, in display order: what Settings and the toolbar popovers both list. */
export interface IRenderChoiceOption<T extends string> {
  label: string;
  value: T;
  /** Whether it is offered but cannot be picked here. */
  isDisabled?: boolean;
}
