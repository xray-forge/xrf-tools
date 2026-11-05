import { ReactNode } from "react";

import { IKeybindCommand } from "@/core/commands";

import { IMenuSection } from "./menu-descriptor";

/** What a menu row is, which decides how it is drawn and what choosing it does. */
export enum EMenuEntryKind {
  COMMAND = "command",
  ACTION = "action",
  TOGGLE = "toggle",
  CHOICE = "choice",
  SUBMENU = "submenu",
}

/**
 * An entry invoking a declared keybind command: its label and chord come from the declaration, and whether it can be
 * chosen from the guard of the handler answering it. Any action that has a chord is published this way.
 */
export interface IMenuCommandEntry {
  readonly kind: EMenuEntryKind.COMMAND;
  readonly command: IKeybindCommand;
  /** Said in place of the command's own label while what it does has turned, as Record stopping a recording. */
  readonly label?: string;
  /** Drawn in place of the command's own icon beside such a label. */
  readonly icon?: ReactNode;
}

/** An entry running a closure: an action with no chord, such as one opening a dialog. */
export interface IMenuActionEntry {
  readonly kind: EMenuEntryKind.ACTION;
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
  readonly kind: EMenuEntryKind.TOGGLE;
  /** `<domain>/<verb-noun>`, as a keybind command's id. */
  readonly id: string;
  readonly label: string;
  readonly isChecked: boolean;
  readonly isDisabled?: boolean;
  /** The command switching the same from the keyboard, whose chord the row shows. */
  readonly command?: IKeybindCommand;
  readonly onToggle: () => void;
}

/** An entry that is one of several exclusive choices, such as what the viewport shows, drawn with a dot when chosen. */
export interface IMenuChoiceEntry {
  readonly kind: EMenuEntryKind.CHOICE;
  /** `<domain>/<verb-noun>`, as a keybind command's id. */
  readonly id: string;
  readonly label: string;
  /** What choosing it shows and how to read it, in the row's tooltip. */
  readonly description?: string;
  readonly isChosen: boolean;
  readonly isDisabled?: boolean;
  readonly onChoose: () => void;
}

/** An entry opening a further menu beside it, which holds its own sections of entries. */
export interface IMenuSubmenuEntry {
  readonly kind: EMenuEntryKind.SUBMENU;
  /** `<domain>/<noun>`, as a menu's id. */
  readonly id: string;
  readonly label: string;
  readonly icon?: ReactNode;
  readonly isDisabled?: boolean;
  /** The further menu's entries, a divider between each run; empty runs are dropped. */
  readonly sections: ReadonlyArray<ReadonlyArray<TMenuEntry>>;
}

/** One row of a menu. */
export type TMenuEntry = IMenuCommandEntry | IMenuActionEntry | IMenuToggleEntry | IMenuChoiceEntry | IMenuSubmenuEntry;

/** Entries one owner publishes into one section, in the order they are drawn. */
export interface IMenuContribution {
  readonly section: IMenuSection;
  readonly entries: ReadonlyArray<TMenuEntry>;
}
