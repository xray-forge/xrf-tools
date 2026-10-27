import { PeopleIcon } from "@xrf/ui/icons/people";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const CHARACTERS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse character profiles and the assets they reference",
    group: EToolGroupId.GAMEPLAY,
    icon: <PeopleIcon />,
    id: EToolId.CHARACTERS_EXPLORER,
    label: "Characters explorer",
    path: "/characters-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
