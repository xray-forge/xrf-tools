import { Container } from "@wirestate/core";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import { VISUAL_INSPECTION } from "@/core/visuals/components/panels/visual-inspection";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { VisualsSequencerApplication as Component } from "./VisualsSequencerApplication";

export const container: ContainerDefinition = {
  bindings: [
    VisualLoadService,
    VisualSequenceService,
    SequencerService,
    // The sequencer's own service answers what the shared inspection panels show.
    { token: VISUAL_INSPECTION, factory: (container: Container) => container.get(SequencerService) },
  ],
};
