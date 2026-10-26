import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { CONFIGS_VERIFIER_HELP } from "./help";

export const CONFIGS_VERIFIER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Validate LTX configuration files",
    group: EApplicationGroupId.CONFIGS,
    help: CONFIGS_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EApplicationId.CONFIGS_VERIFIER,
    label: "Configs verifier",
    path: "/configs-verifier",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
