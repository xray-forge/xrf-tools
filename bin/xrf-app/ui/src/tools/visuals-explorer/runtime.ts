import { Container } from "@wirestate/core";

import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualMotionService } from "@/core/visuals/services/visual-motion";
import { VisualRenderService } from "@/core/visuals/services/visual-render";
import { VisualViewService } from "@/core/visuals/services/visual-view";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { VisualsService } from "@/tools/visuals-explorer/services/visuals";
import { VisualsBrowseService } from "@/tools/visuals-explorer/services/visuals-browse";

export { VisualsExplorerTool as Component } from "./VisualsExplorerTool";

export const container: ContainerDefinition = {
  bindings: [
    VisualLoadService,
    VisualMotionService,
    VisualsService,
    VisualsBrowseService,
    VisualViewService,
    VisualRenderService,
    // Names which service the shared inspection panels read, which is the one thing a tool has to
    // say about them.
    { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(VisualsService) },
    // And which one the viewport draws: the same service, which also says how the picked motion poses it.
    { token: VISUAL_RENDER_SOURCE, factory: (container: Container) => container.get(VisualsService) },
  ],
};
