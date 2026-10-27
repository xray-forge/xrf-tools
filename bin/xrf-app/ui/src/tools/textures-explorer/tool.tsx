import { TextureIcon } from "@xrf/ui/icons/texture";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { TEXTURES_EXPLORER_HELP } from "./help";

export const TEXTURES_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse textures and what their descriptors declare",
    group: EToolGroupId.TEXTURES,
    help: TEXTURES_EXPLORER_HELP,
    icon: <TextureIcon />,
    id: EToolId.TEXTURES_EXPLORER,
    label: "Textures explorer",
    path: "/textures-explorer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
