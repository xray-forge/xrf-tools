import { default as PlaylistPlayIcon } from "@mui/icons-material/PlaylistPlay";
import { lazy } from "react";

import { VisualSequenceService } from "@/applications/visuals-sequencer/services/sequence";
import { SequencerService } from "@/applications/visuals-sequencer/services/sequencer";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";

export const VISUALS_SEQUENCER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./VisualsSequencerApplication").then((it) => ({ default: it.VisualsSequencerApplication }))
  ),
  container: { bindings: [VisualLoadService, VisualSequenceService, SequencerService] },
  preload: () => import("./VisualsSequencerApplication"),
  description: "Compose ordered animation sequences from a visual's motions",
  group: EApplicationGroupId.VISUALS,
  icon: <PlaylistPlayIcon />,
  id: EApplicationId.VISUALS_SEQUENCER,
  label: "Visuals sequencer",
  path: "/visuals-sequencer",
  status: EApplicationStatus.READY,
};
