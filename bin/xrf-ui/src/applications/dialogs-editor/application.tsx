import { default as ForumIcon } from "@mui/icons-material/Forum";
import { lazy } from "react";

import { DIALOGS_EDITOR_HELP } from "@/applications/dialogs-editor/help";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const DIALOGS_EDITOR_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [DialogsService] },
  Component: lazy(() => import("./DialogsEditorApplication").then((it) => ({ default: it.DialogsEditorApplication }))),
  preload: () => import("./DialogsEditorApplication"),
  description: "Browse dialog trees and the lines they resolve to",
  group: EApplicationGroupId.DIALOGS,
  help: DIALOGS_EDITOR_HELP,
  icon: <ForumIcon />,
  id: EApplicationId.DIALOGS_EDITOR,
  label: "Dialogs editor",
  path: "/dialogs-editor",
  status: EApplicationStatus.PLANNED,
};
