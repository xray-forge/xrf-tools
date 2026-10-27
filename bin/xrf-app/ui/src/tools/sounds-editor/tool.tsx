import { TuneIcon } from "@xrf/ui/icons/tune";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SOUNDS_EDITOR_HELP } from "./help";

export const SOUNDS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit sound parameters and prepare game audio",
    group: EToolGroupId.SOUNDS,
    help: SOUNDS_EDITOR_HELP,
    icon: <TuneIcon />,
    id: EToolId.SOUNDS_EDITOR,
    label: "Sounds editor",
    path: "/sounds-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
