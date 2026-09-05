import { default as ForumIcon } from "@mui/icons-material/Forum";

import { DIALOGS_EDITOR_HELP } from "@/applications/dialogs-editor/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const DIALOGS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse dialog trees and the lines they resolve to",
    group: EApplicationGroupId.DIALOGS,
    help: DIALOGS_EDITOR_HELP,
    icon: <ForumIcon />,
    id: EApplicationId.DIALOGS_EDITOR,
    label: "Dialogs editor",
    path: "/dialogs-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
