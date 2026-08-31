import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";
import { lazy } from "react";

import { GAMEDATA_VERIFIER_HELP } from "@/applications/gamedata-verifier/help";
import { GamedataVerifierService } from "@/applications/gamedata-verifier/services/verifier";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const GAMEDATA_VERIFIER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./GamedataVerifierApplication").then((it) => ({ default: it.GamedataVerifierApplication }))
  ),
  container: { bindings: [GamedataVerifierService] },
  preload: () => import("./GamedataVerifierApplication"),
  description: "Run every check over a gamedata tree",
  group: EApplicationGroupId.GAMEDATA,
  help: GAMEDATA_VERIFIER_HELP,
  icon: <FactCheckIcon />,
  id: EApplicationId.GAMEDATA_VERIFIER,
  label: "Gamedata verifier",
  path: "/gamedata-verifier",
  status: EApplicationStatus.READY,
};
