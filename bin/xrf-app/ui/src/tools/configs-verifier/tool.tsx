import { FactCheckIcon } from "@xrf/ui/icons/fact-check";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { CONFIGS_VERIFIER_HELP } from "./help";

export const CONFIGS_VERIFIER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Validate LTX configuration files",
    group: EToolGroupId.CONFIGS,
    help: CONFIGS_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EToolId.CONFIGS_VERIFIER,
    label: "Configs verifier",
    path: "/configs-verifier",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
