import { XrfApplicationError } from "@/core/error/lib";

import { createMenuSections, IMenu, IMenuSection } from "./menu-descriptor";

/**
 * Sections an application adds to a menu it does not declare, such as File, drawn at that menu's extension point while
 * the application is open.
 */
export interface IMenuExtension<S extends string = string> {
  /** The menu the sections go into. */
  readonly menuId: string;
  /** The application adding them, whose slug prefixes each section's id so it never meets the menu's own. */
  readonly owner: string;
  /** Each section by name, which is how a contribution addresses one. */
  readonly sections: { readonly [K in S]: IMenuSection };
  /** The same sections in the order they are drawn. */
  readonly sectionOrder: ReadonlyArray<IMenuSection>;
}

/** An extension as authored. */
export interface IMenuExtensionInit<S extends string> {
  /** The slug of the application adding the sections. */
  owner: string;
  /** Section names in drawing order, kebab case. */
  sections: ReadonlyArray<S>;
}

const OWNER_PATTERN: RegExp = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/**
 * Declares sections an application adds to another menu, whose `sections` are what its contributions name.
 *
 * @param menu - Menu the sections go into.
 * @param init - The application adding them, and their names.
 * @returns The frozen extension.
 */
export function defineMenuExtension<S extends string>(menu: IMenu, init: IMenuExtensionInit<S>): IMenuExtension<S> {
  if (!OWNER_PATTERN.test(init.owner)) {
    throw new XrfApplicationError(`Menu extension owner '${init.owner}' is not a kebab case application slug.`);
  }

  const sections: Record<S, IMenuSection> = createMenuSections(menu.id, init.sections, init.owner);

  return Object.freeze({
    menuId: menu.id,
    owner: init.owner,
    sectionOrder: Object.freeze(init.sections.map((name: S) => sections[name])),
    sections,
  });
}
