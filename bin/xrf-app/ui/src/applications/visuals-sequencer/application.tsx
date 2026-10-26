import { default as PlaylistPlayIcon } from "@mui/icons-material/PlaylistPlay";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { VISUALS_SEQUENCER_HELP } from "./help";

export const VISUALS_SEQUENCER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Compose ordered animation sequences from a visual's motions",
    group: EApplicationGroupId.VISUALS,
    icon: <PlaylistPlayIcon />,
    help: VISUALS_SEQUENCER_HELP,
    id: EApplicationId.VISUALS_SEQUENCER,
    label: "Visuals sequencer",
    path: "/visuals-sequencer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
