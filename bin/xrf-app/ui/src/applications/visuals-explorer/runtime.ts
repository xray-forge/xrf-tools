import { Container } from "@wirestate/core";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { VisualRenderService } from "@/core/visuals/services/visual-render.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { VisualsExplorerApplication as Component } from "./VisualsExplorerApplication";

export const container: ContainerDefinition = {
  bindings: [
    VisualLoadService,
    VisualMotionService,
    VisualsService,
    VisualsBrowseService,
    VisualViewService,
    VisualRenderService,
    // Names which service the shared inspection panels read, which is the one thing an application has to
    // say about them.
    { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(VisualsService) },
    // And which one the viewport draws: the same service, which also says how the picked motion poses it.
    { token: VISUAL_RENDER_SOURCE, factory: (container: Container) => container.get(VisualsService) },
  ],
};
