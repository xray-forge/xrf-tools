import { LibraryMusicIcon } from "@xrf/ui/icons/library-music";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SOUNDS_EXPLORER_HELP } from "./help";

export const SOUNDS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Listen to sounds and inspect their X-Ray metadata",
    group: EToolGroupId.SOUNDS,
    help: SOUNDS_EXPLORER_HELP,
    icon: <LibraryMusicIcon />,
    id: EToolId.SOUNDS_EXPLORER,
    label: "Sounds explorer",
    path: "/sounds-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
