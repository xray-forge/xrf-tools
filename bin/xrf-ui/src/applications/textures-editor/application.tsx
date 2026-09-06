import { default as EditNoteIcon } from "@mui/icons-material/EditNote";

import { TEXTURES_EDITOR_HELP } from "@/applications/textures-editor/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const TEXTURES_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Author and edit the thm descriptor beside a texture",
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
