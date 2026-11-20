import { Nullable } from "@xrf/types";
import { ReactNode } from "react";

/** One row of a menubar's menu that does something when chosen. */
export interface IMenubarItem {
  id: string;
  label: string;
  /** Says more than the label in the row's tooltip, beside the menu. */
  description?: string;
  /** Drawn in the icon column; a switch draws its check there instead. */
  icon?: ReactNode;
  /** Written at the row's end, such as the chord that does the same. */
  shortcut?: Nullable<string>;
  isDisabled?: boolean;
  /** Draws the row as a switch in this state; unset for a plain action. */
  isChecked?: boolean;
  /** Draws the switch as one of several exclusive choices, a dot rather than a check marking the chosen one. */
  isExclusive?: boolean;
  onSelect: () => void;
}

/** One row of a menubar's menu opening a further menu beside it. */
export interface IMenubarSubmenu {
  id: string;
  label: string;
  icon?: ReactNode;
  isDisabled?: boolean;
  /** The further menu's sections, a divider between each. */
  sections: ReadonlyArray<ReadonlyArray<TMenubarEntry>>;
}

/** Any row of a menubar's menu. */
export type TMenubarEntry = IMenubarItem | IMenubarSubmenu;

/** One top-level menu of a menubar. */
export interface IMenubarMenu {
  id: string;
  label: string;
  /** Read when the menu opens, so what a row says and whether it can be chosen is asked then, not kept current. */
  getSections: () => ReadonlyArray<ReadonlyArray<TMenubarEntry>>;
}

/**
 * @param entry - A row of a menu.
 * @returns Whether the row opens a further menu rather than doing something.
 */
export function isMenubarSubmenu(entry: TMenubarEntry): entry is IMenubarSubmenu {
  return "sections" in entry;
}
