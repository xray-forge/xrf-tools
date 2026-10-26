import { default as EditNoteIcon } from "@mui/icons-material/EditNote";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { TEXTURES_EDITOR_HELP } from "./help";

export const TEXTURES_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit texture descriptors, compare encodings and generate bump maps",
    group: EApplicationGroupId.TEXTURES,
    help: TEXTURES_EDITOR_HELP,
    icon: <EditNoteIcon />,
    id: EApplicationId.TEXTURES_EDITOR,
    label: "Textures editor",
    path: "/textures-editor",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
