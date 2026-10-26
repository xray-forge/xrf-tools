import { default as CompareArrowsIcon } from "@mui/icons-material/CompareArrows";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { GAMEDATA_COMPARER_HELP } from "./help";

export const GAMEDATA_COMPARER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Compare installations, mods and effective asset contents",
    group: EApplicationGroupId.GAMEDATA,
    help: GAMEDATA_COMPARER_HELP,
    icon: <CompareArrowsIcon />,
    id: EApplicationId.GAMEDATA_COMPARER,
    label: "Gamedata comparer",
    path: "/gamedata-comparer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
