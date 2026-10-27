import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_DECOMPILER_HELP } from "./help";

export const LEVEL_DECOMPILER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Reconstruct editable scenes from compiled game levels",
    group: EToolGroupId.LEVEL,
    help: LEVEL_DECOMPILER_HELP,
    icon: <AccountTreeIcon />,
    id: EToolId.LEVEL_DECOMPILER,
    label: "Level decompiler",
    path: "/level-decompiler",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
