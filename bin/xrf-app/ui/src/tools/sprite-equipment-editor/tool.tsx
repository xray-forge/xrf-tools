import { ImageIcon } from "@xrf/ui/icons/image";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SPRITE_EQUIPMENT_EDITOR_KEYBIND_COMMANDS } from "./commands";
import { SPRITE_EQUIPMENT_EDITOR_HELP } from "./help";
import { SPRITE_EQUIPMENT_EDITOR_MENUS } from "./menus";

export const SPRITE_EQUIPMENT_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Inspect and edit the icons of an equipment sprite",
    group: EToolGroupId.SPRITES,
    icon: <ImageIcon />,
    help: SPRITE_EQUIPMENT_EDITOR_HELP,
    id: EToolId.SPRITE_EQUIPMENT_EDITOR,
    keybindCommands: SPRITE_EQUIPMENT_EDITOR_KEYBIND_COMMANDS,
    label: "Sprite equipment editor",
    menus: SPRITE_EQUIPMENT_EDITOR_MENUS,
    path: "/sprite-equipment-editor",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
