import { default as TerrainIcon } from "@mui/icons-material/Terrain";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { LEVEL_VIEWER_KEYBIND_COMMANDS } from "./commands";
import { LEVEL_VIEWER_HELP } from "./help";
import { LEVEL_VIEWER_MENU_EXTENSIONS, LEVEL_VIEWER_MENUS } from "./menus";

export const LEVEL_VIEWER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Explore compiled locations with textures and inspection overlays",
    group: EApplicationGroupId.LEVEL,
    help: LEVEL_VIEWER_HELP,
    icon: <TerrainIcon />,
    id: EApplicationId.LEVEL_VIEWER,
    keybindCommands: LEVEL_VIEWER_KEYBIND_COMMANDS,
    label: "Level viewer",
    menuExtensions: LEVEL_VIEWER_MENU_EXTENSIONS,
    menus: LEVEL_VIEWER_MENUS,
    path: "/level-viewer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
