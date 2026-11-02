import { CompareArrowsIcon } from "@xrf/ui/icons/compare-arrows";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { GAMEDATA_COMPARER_HELP } from "./help";

export const GAMEDATA_COMPARER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Compare installations, mods and effective asset contents",
    group: EToolGroupId.GAMEDATA,
    help: GAMEDATA_COMPARER_HELP,
    icon: <CompareArrowsIcon />,
    id: EToolId.GAMEDATA_COMPARER,
    label: "Gamedata comparer",
    path: "/gamedata-comparer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
