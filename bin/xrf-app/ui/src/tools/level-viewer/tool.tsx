import { TerrainIcon } from "@xrf/ui/icons/terrain";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_VIEWER_KEYBIND_COMMANDS } from "./commands";
import { LEVEL_VIEWER_HELP } from "./help";
import { LEVEL_VIEWER_MENU_EXTENSIONS, LEVEL_VIEWER_MENUS } from "./menus";

export const LEVEL_VIEWER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Explore compiled locations with textures and inspection overlays",
    group: EToolGroupId.LEVEL,
    help: LEVEL_VIEWER_HELP,
    icon: <TerrainIcon />,
    id: EToolId.LEVEL_VIEWER,
    keybindCommands: LEVEL_VIEWER_KEYBIND_COMMANDS,
    label: "Level viewer",
    menuExtensions: LEVEL_VIEWER_MENU_EXTENSIONS,
    menus: LEVEL_VIEWER_MENUS,
    path: "/level-viewer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
