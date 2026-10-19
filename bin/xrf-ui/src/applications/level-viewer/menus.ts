import { defineMenu, IMenu } from "@/core/shell/menu/lib";

/** What the level viewer draws over the open level, switched from the menu as from the toolbar's overlays. */
export const LEVEL_VIEWER_VIEW_MENU: IMenu<"overlays"> = defineMenu({
  id: "level-viewer/view",
  label: "View",
  sections: ["overlays"],
});

/** Menus the level viewer adds between File and Help. */
export const LEVEL_VIEWER_MENUS: ReadonlyArray<IMenu> = [LEVEL_VIEWER_VIEW_MENU];
