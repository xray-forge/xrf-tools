import { EToolId } from "@/core/routing/tool";
import { defineMenu, defineMenuExtension, IMenu, IMenuExtension } from "@/core/shell/menu/lib";
import { FILE_MENU } from "@/core/shell/menu/menus";

/** What the level viewer draws over the open level, switched from the menu as from the toolbar's overlays. */
export const LEVEL_VIEWER_VIEW_MENU: IMenu<"overlays"> = defineMenu({
  id: "level-viewer/view",
  label: "View",
  sections: ["overlays"],
});

/** What the level viewer adds to File: the view written to a file and read back, and the screenshot. */
export const LEVEL_VIEWER_FILE_MENU_EXTENSION: IMenuExtension<"view" | "screenshot"> = defineMenuExtension(FILE_MENU, {
  owner: EToolId.LEVEL_VIEWER,
  sections: ["view", "screenshot"],
});

/** Sections the level viewer adds to the core menus. */
export const LEVEL_VIEWER_MENU_EXTENSIONS: ReadonlyArray<IMenuExtension> = [LEVEL_VIEWER_FILE_MENU_EXTENSION];

/** Menus the level viewer adds between File and Help. */
export const LEVEL_VIEWER_MENUS: ReadonlyArray<IMenu> = [LEVEL_VIEWER_VIEW_MENU];
