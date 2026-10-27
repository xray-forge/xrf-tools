import { ArchiveIcon } from "@xrf/ui/icons/archive";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { ARCHIVES_PACKER_HELP } from "./help";

export const ARCHIVES_PACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Pack a directory into game archives",
    group: EToolGroupId.ARCHIVES,
    help: ARCHIVES_PACKER_HELP,
    icon: <ArchiveIcon />,
    id: EToolId.ARCHIVES_PACKER,
    label: "Archives packer",
    path: "/archives-packer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
