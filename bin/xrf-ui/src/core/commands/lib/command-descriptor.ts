import { ReactNode } from "react";

import { XrfApplicationError } from "@/core/error/lib";

/**
 * The family a command belongs to, used to group it in listings.
 */
export enum ECommandCategory {
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
export const COMMAND_CATEGORY_LABELS: Readonly<Record<ECommandCategory, string>> = {
  [ECommandCategory.APPLICATION]: "Application",
  [ECommandCategory.NAVIGATION]: "Navigation",
  [ECommandCategory.EDITING]: "Editing",
  [ECommandCategory.VIEW]: "View",
  [ECommandCategory.HELP]: "Help",
};

/**
 * `<domain>/<verb-noun>`, where the domain is the `<subject>-<role>` slug that already names the directory, route,
 * launcher entry and tests. An id therefore greps straight back to its owner, and two applications may both declare
 * a `repack` without colliding.
 */
const COMMAND_ID_PATTERN: RegExp = /^[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*$/;

/** Identity and presentation of one command, independent of anything that invokes or implements it. */
export interface ICommandDescriptor {
  readonly id: string;
  /** Imperative and short, as it appears in a palette row or a shortcuts table. */
  readonly label: string;
  readonly description: string;
  readonly category: ECommandCategory;
  /** Optional, for surfaces that draw commands rather than list them. */
  readonly icon?: ReactNode;
  /**
   * Default chords, in preference order. Inert here: `core/commands` carries the strings and reads none of them,
   * while `core/keybinds` owns every parse, normalization and match.
   */
  readonly chords: ReadonlyArray<string>;
}

/** A descriptor as authored, before defaults are filled in. */
export interface ICommandDescriptorInit extends Omit<ICommandDescriptor, "chords"> {
  chords?: ReadonlyArray<string>;
}

/**
 * Declares one command.
 *
 * The returned descriptor is the token: `@Command(TOKEN)` takes it rather than an id, so a handler cannot bind a
 * command nobody declared, and go-to-definition from a handler or a shortcuts row lands on this label.
 *
 * @param init - Identity and presentation of the command.
 * @returns The frozen descriptor, used as the token everywhere else.
 */
export function defineCommand(init: ICommandDescriptorInit): ICommandDescriptor {
  if (!COMMAND_ID_PATTERN.test(init.id)) {
    throw new XrfApplicationError(`Command id '${init.id}' is not '<domain>/<verb-noun>' in kebab case.`);
  }

  return Object.freeze({ ...init, chords: Object.freeze([...(init.chords ?? [])]) });
}

/**
 * Query type carrying one command's enablement.
 *
 * @param descriptor - Command whose guard is addressed.
 * @returns The query type the guard registers and `CommandsService` asks.
 */
export function toCommandEnabledQuery(descriptor: ICommandDescriptor): string {
  return `${descriptor.id}#enabled`;
}
