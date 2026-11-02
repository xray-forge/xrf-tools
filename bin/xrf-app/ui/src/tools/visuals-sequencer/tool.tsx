import { PlaylistPlayIcon } from "@xrf/ui/icons/playlist-play";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { VISUALS_SEQUENCER_HELP } from "./help";

export const VISUALS_SEQUENCER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Compose ordered animation sequences from a visual's motions",
    group: EToolGroupId.VISUALS,
    icon: <PlaylistPlayIcon />,
    help: VISUALS_SEQUENCER_HELP,
    id: EToolId.VISUALS_SEQUENCER,
    label: "Visuals sequencer",
    path: "/visuals-sequencer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
