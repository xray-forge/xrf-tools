import { Nullable } from "@xrf/types";

import { IMenu, IMenuSection, isSameMenuSection } from "./menu-descriptor";
import { IMenuContribution, TMenuEntry } from "./menu-entry";
import { IMenuExtension } from "./menu-extension";

/** One menu as the bar draws it: its non-empty sections, each holding its entries in publication order. */
export interface IAssembledMenu {
  readonly menu: IMenu;
  readonly sections: ReadonlyArray<ReadonlyArray<TMenuEntry>>;
}

/**
 * Lays published entries out in declared section order, dropping empty sections and menus.
 *
 * @param menus - The bar's menus, in bar order.
 * @param extensions - Sections the open application adds to those menus, drawn at each one's extension point.
 * @param contributions - Everything published, in publication order.
 * @returns The menus that have something to show.
 */
export function assembleMenus(
  menus: ReadonlyArray<IMenu>,
  extensions: ReadonlyArray<IMenuExtension>,
  contributions: ReadonlyArray<IMenuContribution>
): Array<IAssembledMenu> {
  const assembled: Array<IAssembledMenu> = [];

  for (const menu of menus) {
    const sections: Array<ReadonlyArray<TMenuEntry>> = [];

    for (const section of toSectionOrder(menu, extensions)) {
      const entries: Array<TMenuEntry> = contributions
        .filter((it: IMenuContribution) => isSameMenuSection(it.section, section))
        .flatMap((it: IMenuContribution) => it.entries);

      if (entries.length > 0) {
        sections.push(entries);
      }
    }

    if (sections.length > 0) {
      assembled.push({ menu, sections });
    }
  }

  return assembled;
}

/**
 * A menu's sections with those added to it spliced in at its extension point, in the order they were declared.
 *
 * @param menu - Menu to order.
 * @param extensions - Every extension in force, of any menu.
 * @returns The sections the menu draws, in order.
 */
function toSectionOrder(menu: IMenu, extensions: ReadonlyArray<IMenuExtension>): ReadonlyArray<IMenuSection> {
  const added: Array<IMenuSection> = extensions
    .filter((it: IMenuExtension) => it.menuId === menu.id)
    .flatMap((it: IMenuExtension) => it.sectionOrder);

  if (added.length === 0) {
    return menu.sectionOrder;
  }

  const point: Nullable<IMenuSection> = menu.extensionPoint;
  const at: number = point
    ? menu.sectionOrder.findIndex((it: IMenuSection) => isSameMenuSection(it, point))
    : menu.sectionOrder.length;

  return [...menu.sectionOrder.slice(0, at), ...added, ...menu.sectionOrder.slice(at)];
}
