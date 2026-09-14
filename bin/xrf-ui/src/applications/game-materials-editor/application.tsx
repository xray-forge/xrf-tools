import { default as LayersIcon } from "@mui/icons-material/Layers";

import { GAME_MATERIALS_EDITOR_HELP } from "@/applications/game-materials-editor/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const GAME_MATERIALS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit physical materials and their interaction pairs",
    group: EApplicationGroupId.MATERIALS,
    help: GAME_MATERIALS_EDITOR_HELP,
    icon: <LayersIcon />,
    id: EApplicationId.GAME_MATERIALS_EDITOR,
    label: "Game materials editor",
    path: "/game-materials-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
