import { XrfApplicationError } from "@xrf/errors";
import { Nullable } from "@xrf/types";

/** One ordered run of entries inside a menu, which entries are published into. */
export interface IMenuSection {
  readonly id: string;
  /** The menu declaring this section, where an entry published into it is drawn. */
  readonly menuId: string;
}

/** One top-level menu of the window's menubar: its name, and the sections its entries are published into. */
export interface IMenu<S extends string = string> {
  readonly id: string;
  readonly label: string;
  /** Each section by name, which is how a contribution addresses one. */
  readonly sections: { readonly [K in S]: IMenuSection };
  /** The same sections in the order the menu draws them. */
  readonly sectionOrder: ReadonlyArray<IMenuSection>;
  /** Where sections a tool adds to this menu are drawn: before this one, or after the last when null. */
  readonly extensionPoint: Nullable<IMenuSection>;
}

/** A menu as authored. */
export interface IMenuInit<S extends string> {
  /** `<domain>/<noun>`, where the domain is the slug of the directory declaring it, as a keybind command's id. */
  id: string;
  label: string;
  /** Section names in drawing order, kebab case. */
  sections: ReadonlyArray<S>;
  /** The section that sections added by a tool go before; they go after the last when omitted. */
  extensionsBefore?: S;
}

const MENU_ID_PATTERN: RegExp = /^[a-z0-9]+(?:-[a-z0-9]+)*\/[a-z0-9]+(?:-[a-z0-9]+)*$/;

const SECTION_ID_PATTERN: RegExp = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/**
 * Declares one top-level menu, whose `sections` are what contributions name.
 *
 * @param init - Identity, name and sections of the menu.
 * @returns The frozen descriptor.
 */
export function defineMenu<S extends string>(init: IMenuInit<S>): IMenu<S> {
  if (!MENU_ID_PATTERN.test(init.id)) {
    throw new XrfApplicationError(`Menu id '${init.id}' is not '<domain>/<noun>' in kebab case.`);
  }

  const sections: Record<S, IMenuSection> = createMenuSections(init.id, init.sections);

  return Object.freeze({
    id: init.id,
    label: init.label,
    sectionOrder: Object.freeze(init.sections.map((name: S) => sections[name])),
    sections,
    extensionPoint: init.extensionsBefore ? sections[init.extensionsBefore] : null,
  });
}

/**
 * Creates the sections one declaration adds to a menu, checking their names.
 *
 * @param menuId - Menu the sections belong to.
 * @param names - Section names in drawing order, kebab case.
 * @param owner - Slug prefixing each id, for sections a tool adds to a menu it does not declare.
 * @returns The frozen sections by name.
 */
export function createMenuSections<S extends string>(
  menuId: string,
  names: ReadonlyArray<S>,
  owner: Nullable<string> = null
): Record<S, IMenuSection> {
  if (names.length === 0) {
    throw new XrfApplicationError(`Menu '${menuId}' is given no section, so nothing could be published into it.`);
  }

  const sections: Partial<Record<S, IMenuSection>> = {};

  for (const name of names) {
    if (!SECTION_ID_PATTERN.test(name)) {
      throw new XrfApplicationError(`Menu '${menuId}' names section '${name}', which is not kebab case.`);
    }

    if (sections[name]) {
      throw new XrfApplicationError(`Menu '${menuId}' names section '${name}' twice.`);
    }

    sections[name] = Object.freeze({ id: owner ? `${owner}/${name}` : name, menuId });
  }

  return Object.freeze(sections as Record<S, IMenuSection>);
}

/**
 * Whether two sections are the same one, by identity rather than by reference.
 *
 * @param first - One section.
 * @param second - The other.
 * @returns Whether both name the same section of the same menu.
 */
export function isSameMenuSection(first: IMenuSection, second: IMenuSection): boolean {
  return first.menuId === second.menuId && first.id === second.id;
}
