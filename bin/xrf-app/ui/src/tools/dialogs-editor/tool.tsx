import { ForumIcon } from "@xrf/ui/icons/forum";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { DIALOGS_EDITOR_HELP } from "./help";

export const DIALOGS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse dialog trees and the lines they resolve to",
    group: EToolGroupId.DIALOGS,
    help: DIALOGS_EDITOR_HELP,
    icon: <ForumIcon />,
    id: EToolId.DIALOGS_EDITOR,
    label: "Dialogs editor",
    path: "/dialogs-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
