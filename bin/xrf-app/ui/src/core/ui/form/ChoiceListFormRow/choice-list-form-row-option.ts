import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** One value a choice list offers, which may name the section it is listed under and say what it is. */
export interface IChoiceListFormRowOption<T extends string> extends IChoiceFormRowOption<T> {
  /** The heading it is listed under, drawn once over a run of neighbours sharing it. */
  section?: string;
  /** One line on what it is, its tooltip in the list and said under the list while it is chosen. */
  description?: string;
}
