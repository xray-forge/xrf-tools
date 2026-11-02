import { Nullable } from "@xrf/types";
import { ReactNode } from "react";

/** One row of a menubar's menu. */
export interface IMenubarItem {
  id: string;
  label: string;
  /** Drawn in the icon column; a switch draws its check there instead. */
  icon?: ReactNode;
  /** Written at the row's end, such as the chord that does the same. */
  shortcut?: Nullable<string>;
  isDisabled?: boolean;
  /** Draws the row as a switch in this state; unset for a plain action. */
  isChecked?: boolean;
  onSelect: () => void;
}

/** One top-level menu of a menubar. */
export interface IMenubarMenu {
  id: string;
  label: string;
  /** Read when the menu opens, so what a row says and whether it can be chosen is asked then, not kept current. */
  getSections: () => ReadonlyArray<ReadonlyArray<IMenubarItem>>;
}
