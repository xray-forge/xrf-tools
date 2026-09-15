import { ReactNode } from "react";

import { XrfApplicationError } from "@/core/error/lib";

/**
 * The family a keybind command belongs to, used to group it in listings.
 */
export enum EKeybindCommandCategory {
  /** Opening, closing, repacking - whatever the tool as a whole does. */
  APPLICATION = "application",
  /** Changes to the open document. */
  EDITING = "editing",
  HELP = "help",
  /** Moving between tools, documents and panels. */
  NAVIGATION = "navigation",
  /** What is drawn, without changing it. */
  VIEW = "view",
}

/**
 * Heading each category is listed under, and the order listings show them in.
 *
 * Separate from the id for the reason `IApplicationGroup` separates its own: an id is written into declarations and
 * compared, a heading is read, and the two should be free to change independently.
 */
export const KEYBIND_COMMAND_CATEGORY_LABELS: Readonly<Record<EKeybindCommandCategory, string>> = {
  [EKeybindCommandCategory.APPLICATION]: "Application",
  [EKeybindCommandCategory.NAVIGATION]: "Navigation",
  [EKeybindCommandCategory.EDITING]: "Editing",
  [EKeybindCommandCategory.VIEW]: "View",
  [EKeybindCommandCategory.HELP]: "Help",
};

/**
 * `<domain>/<verb-noun>`, where the domain is the `<subject>-<role>` slug that already names the directory, route,
 * launcher entry and tests. An id therefore greps straight back to its owner, and two applications may both declare
 * a `repack` without colliding.
 */
const KEYBIND_COMMAND_ID_PATTERN: RegExp = /^[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*$/;

/**
 * Identity and presentation of one keybind command, independent of anything that invokes or implements it.
 *
 * A keybind command is what a person invokes: a chord today, a toolbar button or a palette row later. Distinct from
 * a `CommandBus` message, which services send each other and no listing ever shows.
 */
export interface IKeybindCommand {
  readonly id: string;
  /** Imperative and short, as it appears in a palette row or a shortcuts table. */
  readonly label: string;
  readonly description: string;
  readonly category: EKeybindCommandCategory;
  /** Optional, for surfaces that draw commands rather than list them. */
  readonly icon?: ReactNode;
  /**
   * Default chords, in preference order. Inert here: `core/commands` carries the strings and reads none of them,
   * while `core/keybinds` owns every parse, normalization and match.
   */
  readonly chords: ReadonlyArray<string>;
}

/** A descriptor as authored, before defaults are filled in. */
export interface IKeybindCommandInit extends Omit<IKeybindCommand, "chords"> {
  chords?: ReadonlyArray<string>;
}

/**
 * Declares one keybind command.
 *
 * The returned descriptor is the token: `@KeybindCommand(TOKEN)` takes it rather than an id, so a handler cannot bind a
 * command nobody declared, and go-to-definition from a handler or a shortcuts row lands on this label.
 *
 * @param init - Identity and presentation of the command.
 * @returns The frozen descriptor, used as the token everywhere else.
 */
export function defineKeybindCommand(init: IKeybindCommandInit): IKeybindCommand {
  if (!KEYBIND_COMMAND_ID_PATTERN.test(init.id)) {
    throw new XrfApplicationError(`Keybind command id '${init.id}' is not '<domain>/<verb-noun>' in kebab case.`);
  }

  return Object.freeze({ ...init, chords: Object.freeze([...(init.chords ?? [])]) });
}

/**
 * Query type carrying one keybind command's enablement.
 *
 * @param descriptor - Command whose guard is addressed.
 * @returns The query type the guard registers and `KeybindCommandsService` asks.
 */
export function toKeybindCommandEnabledQuery(descriptor: IKeybindCommand): string {
  return `${descriptor.id}#enabled`;
}
