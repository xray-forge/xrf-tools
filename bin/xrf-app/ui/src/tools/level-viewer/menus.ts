import { EToolId } from "@/core/routing/tool";
import { defineMenu, defineMenuExtension, IMenu, IMenuExtension } from "@/core/shell/menu/lib";
import { FILE_MENU } from "@/core/shell/menu/menus";

/**
 * How the level viewer shows the open level, switched from the menu as from the toolbar: the frame (its shading and
 * the overlays over it), then the level objects (their helpers and which are drawn).
 */
export const LEVEL_VIEWER_VIEW_MENU: IMenu<"frame" | "objects"> = defineMenu({
  id: "level-viewer/view",
  label: "View",
  sections: ["frame", "objects"],
});

/** What the level viewer does with the open level: Capture ▸ first, then where the camera goes. */
export const LEVEL_VIEWER_ACTIONS_MENU: IMenu<"capture" | "camera"> = defineMenu({
  id: "level-viewer/actions",
  label: "Actions",
  sections: ["capture", "camera"],
});

/** What the level viewer adds to File: the view written to a file and read back. */
export const LEVEL_VIEWER_FILE_MENU_EXTENSION: IMenuExtension<"view"> = defineMenuExtension(FILE_MENU, {
  owner: EToolId.LEVEL_VIEWER,
  sections: ["view"],
});

/** Sections the level viewer adds to the core menus. */
export const LEVEL_VIEWER_MENU_EXTENSIONS: ReadonlyArray<IMenuExtension> = [LEVEL_VIEWER_FILE_MENU_EXTENSION];

/** Menus the level viewer adds between File and Help. */
export const LEVEL_VIEWER_MENUS: ReadonlyArray<IMenu> = [LEVEL_VIEWER_VIEW_MENU, LEVEL_VIEWER_ACTIONS_MENU];
