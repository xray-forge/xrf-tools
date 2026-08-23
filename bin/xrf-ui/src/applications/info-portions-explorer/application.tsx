import { default as HubIcon } from "@mui/icons-material/Hub";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const INFO_PORTIONS_EXPLORER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./InfoPortionsExplorerApplication").then((it) => ({ default: it.InfoPortionsExplorerApplication }))
  ),
  preload: () => import("./InfoPortionsExplorerApplication"),
  description: "Browse info portions and what gives or requires them",
  group: EApplicationGroupId.GAMEPLAY,
  icon: <HubIcon />,
  id: EApplicationId.INFO_PORTIONS_EXPLORER,
  label: "Info portions explorer",
  path: "/info-portions-explorer",
  status: EApplicationStatus.PLANNED,
};
