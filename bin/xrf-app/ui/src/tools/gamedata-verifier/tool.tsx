import { FactCheckIcon } from "@xrf/ui/icons/fact-check";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { GAMEDATA_VERIFIER_HELP } from "./help";

export const GAMEDATA_VERIFIER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Run every check over a gamedata tree",
    group: EToolGroupId.GAMEDATA,
    help: GAMEDATA_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EToolId.GAMEDATA_VERIFIER,
    label: "Gamedata verifier",
    path: "/gamedata-verifier",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
