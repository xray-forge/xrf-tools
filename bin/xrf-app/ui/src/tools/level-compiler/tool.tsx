import { BuildIcon } from "@xrf/ui/icons/build";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_COMPILER_HELP } from "./help";

export const LEVEL_COMPILER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Compile editable level scenes into game-ready locations",
    group: EToolGroupId.LEVEL,
    help: LEVEL_COMPILER_HELP,
    icon: <BuildIcon />,
    id: EToolId.LEVEL_COMPILER,
    label: "Level compiler",
    path: "/level-compiler",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
