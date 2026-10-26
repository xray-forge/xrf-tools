import { defineMenu, IMenu } from "@/core/shell/menu/lib";

/** What the editor does with the open sprite as a whole. */
export const SPRITE_EQUIPMENT_EDITOR_MENU: IMenu<"sprite"> = defineMenu({
  id: "sprite-equipment-editor/sprite",
  label: "Sprite",
  sections: ["sprite"],
});

/** Menus the equipment sprite editor adds between File and Help. */
export const SPRITE_EQUIPMENT_EDITOR_MENUS: ReadonlyArray<IMenu> = [SPRITE_EQUIPMENT_EDITOR_MENU];
