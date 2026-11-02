import { UnarchiveIcon } from "@xrf/ui/icons/unarchive";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { ARCHIVES_UNPACKER_HELP } from "./help";

export const ARCHIVES_UNPACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Extract game archives into a directory",
    group: EToolGroupId.ARCHIVES,
    help: ARCHIVES_UNPACKER_HELP,
    icon: <UnarchiveIcon />,
    id: EToolId.ARCHIVES_UNPACKER,
    label: "Archives unpacker",
    path: "/archives-unpacker",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
