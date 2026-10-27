import { Container } from "@wirestate/core";

import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualRenderService } from "@/core/visuals/services/visual-render";
import { VisualViewService } from "@/core/visuals/services/visual-view";
import { ContainerDefinition } from "@/lib/container/container-definition";
import { VisualSequenceService } from "@/tools/visuals-sequencer/services/visual-sequence";
import { VisualsSequencerService } from "@/tools/visuals-sequencer/services/visuals-sequencer";

export { VisualsSequencerTool as Component } from "./VisualsSequencerTool";

export const container: ContainerDefinition = {
  bindings: [
    VisualLoadService,
    VisualSequenceService,
    VisualsSequencerService,
    VisualViewService,
    VisualRenderService,
    // The sequencer's own service answers what the shared inspection panels show.
    { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(VisualsSequencerService) },
    // And what the viewport draws: the same service, posed by the track rather than by a picked motion.
    { token: VISUAL_RENDER_SOURCE, factory: (container: Container) => container.get(VisualsSequencerService) },
  ],
};
