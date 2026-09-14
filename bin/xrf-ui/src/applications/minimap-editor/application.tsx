import { default as MapIcon } from "@mui/icons-material/Map";

import { MINIMAP_EDITOR_HELP } from "@/applications/minimap-editor/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const MINIMAP_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Capture level minimaps and edit their world bounds",
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
