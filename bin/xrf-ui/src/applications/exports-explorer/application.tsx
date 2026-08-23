import { default as SwapHorizIcon } from "@mui/icons-material/SwapHoriz";
import { lazy } from "react";

import { ExportsService } from "@/applications/exports-explorer/services/exports";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const EXPORTS_EXPLORER_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [ExportsService] },
  Component: lazy(() =>
    import("./ExportsExplorerApplication").then((it) => ({ default: it.ExportsExplorerApplication }))
  ),
  preload: () => import("@/applications/exports-explorer/ExportsExplorerApplication"),
  description: "Browse TypeScript extern declarations in an XRF project",
  group: EApplicationGroupId.EXPORTS,
  icon: <SwapHorizIcon />,
  id: EApplicationId.EXPORTS_EXPLORER,
  label: "Exports explorer",
  path: "/exports-explorer",
  status: EApplicationStatus.READY,
};
