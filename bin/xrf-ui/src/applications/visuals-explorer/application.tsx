import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Container } from "@wirestate/core";
import { lazy } from "react";

import { VISUALS_EXPLORER_HELP } from "@/applications/visuals-explorer/help";
import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";

export const VISUALS_EXPLORER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./VisualsExplorerApplication").then((it) => ({ default: it.VisualsExplorerApplication }))
  ),
  container: {
    bindings: [
      VisualLoadService,
      VisualMotionService,
      VisualsService,
      VisualsBrowseService,
      // Names which service the shared inspection panels read, which is the one thing an application has to
      // say about them.
      { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(VisualsService) },
    ],
  },
  preload: () => import("./VisualsExplorerApplication"),
  description: "Browse and preview game visuals in 3D",
  group: EApplicationGroupId.VISUALS,
  help: VISUALS_EXPLORER_HELP,
  icon: <ViewInArIcon />,
  id: EApplicationId.VISUALS_EXPLORER,
  label: "Visuals explorer",
  path: "/visuals-explorer",
  status: EApplicationStatus.READY,
};
