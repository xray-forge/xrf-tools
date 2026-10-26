import { Container } from "@wirestate/core";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualRenderService } from "@/core/visuals/services/visual-render.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { VisualsSequencerApplication as Component } from "./VisualsSequencerApplication";

export const container: ContainerDefinition = {
  bindings: [
    VisualLoadService,
    VisualSequenceService,
    SequencerService,
    VisualViewService,
    VisualRenderService,
    // The sequencer's own service answers what the shared inspection panels show.
    { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(SequencerService) },
    // And what the viewport draws: the same service, posed by the track rather than by a picked motion.
    { token: VISUAL_RENDER_SOURCE, factory: (container: Container) => container.get(SequencerService) },
  ],
};
