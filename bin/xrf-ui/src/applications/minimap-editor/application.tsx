import { default as MapIcon } from "@mui/icons-material/Map";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { MINIMAP_EDITOR_HELP } from "./help";

export const MINIMAP_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Align a level map image to its level and edit its world bounds",
    group: EApplicationGroupId.LEVEL,
    help: MINIMAP_EDITOR_HELP,
    icon: <MapIcon />,
    id: EApplicationId.MINIMAP_EDITOR,
    label: "Minimap editor",
    path: "/minimap-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
