import { DifferenceIcon } from "@xrf/ui/icons/difference";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { ARCHIVES_PATCHER_HELP } from "./help";

export const ARCHIVES_PATCHER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Pack gamedata changes as a patch",
    group: EToolGroupId.ARCHIVES,
    help: ARCHIVES_PATCHER_HELP,
    icon: <DifferenceIcon />,
    id: EToolId.ARCHIVES_PATCHER,
    label: "Archives patcher",
    path: "/archives-patcher",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
