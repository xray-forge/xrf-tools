import { EditNoteIcon } from "@xrf/ui/icons/edit-note";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TEXTURES_EDITOR_HELP } from "./help";

export const TEXTURES_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit texture descriptors, compare encodings and generate bump maps",
    group: EToolGroupId.TEXTURES,
    help: TEXTURES_EDITOR_HELP,
    icon: <EditNoteIcon />,
    id: EToolId.TEXTURES_EDITOR,
    label: "Textures editor",
    path: "/textures-editor",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
