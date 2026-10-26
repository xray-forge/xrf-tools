import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { GAMEDATA_VERIFIER_HELP } from "./help";

export const GAMEDATA_VERIFIER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Run every check over a gamedata tree",
    group: EApplicationGroupId.GAMEDATA,
    help: GAMEDATA_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EApplicationId.GAMEDATA_VERIFIER,
    label: "Gamedata verifier",
    path: "/gamedata-verifier",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
