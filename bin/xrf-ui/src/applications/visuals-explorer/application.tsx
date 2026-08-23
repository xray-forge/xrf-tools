import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { lazy } from "react";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";

export const VISUALS_EXPLORER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./VisualsExplorerApplication").then((it) => ({ default: it.VisualsExplorerApplication }))
  ),
  container: { bindings: [VisualLoadService, VisualMotionService, VisualsService, VisualsBrowseService] },
  preload: () => import("./VisualsExplorerApplication"),
  description: "Browse and preview game visuals in 3D",
  group: EApplicationGroupId.VISUALS,
  icon: <ViewInArIcon />,
  id: EApplicationId.VISUALS_EXPLORER,
  label: "Visuals explorer",
  path: "/visuals-explorer",
  status: EApplicationStatus.READY,
};
