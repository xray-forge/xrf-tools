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

export const LEVEL_VIEWER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Explore compiled locations with textures and inspection overlays",
    group: EApplicationGroupId.LEVEL,
    help: LEVEL_VIEWER_HELP,
    icon: <TerrainIcon />,
    id: EApplicationId.LEVEL_VIEWER,
    keybindCommands: LEVEL_VIEWER_KEYBIND_COMMANDS,
    label: "Level viewer",
    path: "/level-viewer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
