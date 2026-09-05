import { Container } from "@wirestate/core";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { VisualsExplorerApplication as Component } from "./VisualsExplorerApplication";

export const container: ContainerDefinition = {
  bindings: [
    VisualLoadService,
    VisualMotionService,
    VisualsService,
    VisualsBrowseService,
    // Names which service the shared inspection panels read, which is the one thing an application has to
    // say about them.
    { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(VisualsService) },
  ],
};
