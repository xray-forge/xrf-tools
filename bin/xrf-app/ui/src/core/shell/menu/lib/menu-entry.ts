import { ReactNode } from "react";

import { IKeybindCommand } from "@/core/commands";

import { IMenuSection } from "./menu-descriptor";

/**
 * An entry invoking a declared keybind command: its label and chord come from the declaration, and whether it can be
 * chosen from the guard of the handler answering it. Any action that has a chord is published this way.
 */
export interface IMenuCommandEntry {
  readonly kind: "command";
  readonly command: IKeybindCommand;
}

/** An entry running a closure: an action with no chord, such as one opening a dialog. */
export interface IMenuActionEntry {
  readonly kind: "action";
  /** `<domain>/<verb-noun>`, as a keybind command's id. */
  readonly id: string;
  /** Ends in "…" when choosing it asks for more before anything happens. */
  readonly label: string;
  /** Drawn in the menu's icon column, as a keybind command's own icon is. */
  readonly icon?: ReactNode;
  readonly isDisabled?: boolean;
  readonly onSelect: () => void;
}

/** An entry switching something on or off, drawn with its state. */
export interface IMenuToggleEntry {
  readonly kind: "toggle";
  /** `<domain>/<verb-noun>`, as a keybind command's id. */
  readonly id: string;
  readonly label: string;
  readonly isChecked: boolean;
  readonly isDisabled?: boolean;
  readonly onToggle: () => void;
}

/** One row of a menu. */
export type TMenuEntry = IMenuCommandEntry | IMenuActionEntry | IMenuToggleEntry;

/** Entries one owner publishes into one section, in the order they are drawn. */
export interface IMenuContribution {
  readonly section: IMenuSection;
  readonly entries: ReadonlyArray<TMenuEntry>;
}
