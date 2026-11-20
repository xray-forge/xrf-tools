import { EToolId } from "@/core/routing/tool";

import { createMenuSections, IMenu, IMenuSection } from "./menu-descriptor";

/**
 * Sections a tool adds to a menu it does not declare, such as File, drawn at that menu's extension point while
 * the tool is open.
 */
export interface IMenuExtension<S extends string = string> {
  /** The menu the sections go into. */
  readonly menuId: string;
  /** The tool adding them, whose id prefixes each section's id so it never meets the menu's own. */
  readonly owner: EToolId;
  /** Each section by name, which is how a contribution addresses one. */
  readonly sections: { readonly [K in S]: IMenuSection };
  /** The same sections in the order they are drawn. */
  readonly sectionOrder: ReadonlyArray<IMenuSection>;
}

/** An extension as authored. */
export interface IMenuExtensionInit<S extends string> {
  /** The tool adding the sections. */
  owner: EToolId;
  /** Section names in drawing order, kebab case. */
  sections: ReadonlyArray<S>;
}

/**
 * Declares sections a tool adds to another menu, whose `sections` are what its contributions name.
 *
 * @param menu - Menu the sections go into.
 * @param init - The tool adding them, and their names.
 * @returns The frozen extension.
 */
export function defineMenuExtension<S extends string>(menu: IMenu, init: IMenuExtensionInit<S>): IMenuExtension<S> {
  const sections: Record<S, IMenuSection> = createMenuSections(menu.id, init.sections, init.owner);

  return Object.freeze({
    menuId: menu.id,
    owner: init.owner,
    sectionOrder: Object.freeze(init.sections.map((name: S) => sections[name])),
    sections,
  });
}
