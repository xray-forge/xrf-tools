import { defineMenu, IMenu } from "@/core/routing/menu-descriptor";
import { defineMenuExtension, IMenuExtension } from "@/core/routing/menu-extension";
import { EToolId } from "@/core/routing/tool";
import { FILE_MENU } from "@/core/shell/menu/menus";

/**
 * The level viewer's View menu: the frame's shading and overlays, then the level objects' helpers and visibility.
 */
export const LEVEL_VIEWER_VIEW_MENU: IMenu<"frame" | "objects"> = defineMenu({
  id: "level-viewer/view",
  label: "View",
  sections: ["frame", "objects"],
});

/** The level viewer's Actions menu: Capture ▸, then camera placement. */
export const LEVEL_VIEWER_ACTIONS_MENU: IMenu<"capture" | "camera"> = defineMenu({
  id: "level-viewer/actions",
  label: "Actions",
  sections: ["capture", "camera"],
});

/** The level viewer's File section: view export and import. */
export const LEVEL_VIEWER_FILE_MENU_EXTENSION: IMenuExtension<"view"> = defineMenuExtension(FILE_MENU, {
  owner: EToolId.LEVEL_VIEWER,
  sections: ["view"],
});

/** Sections the level viewer adds to the core menus. */
export const LEVEL_VIEWER_MENU_EXTENSIONS: ReadonlyArray<IMenuExtension> = [LEVEL_VIEWER_FILE_MENU_EXTENSION];

/** Menus the level viewer adds between File and Help. */
export const LEVEL_VIEWER_MENUS: ReadonlyArray<IMenu> = [LEVEL_VIEWER_VIEW_MENU, LEVEL_VIEWER_ACTIONS_MENU];
