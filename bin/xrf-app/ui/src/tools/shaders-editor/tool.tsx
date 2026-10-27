import { PaletteIcon } from "@xrf/ui/icons/palette";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SHADERS_EDITOR_HELP } from "./help";

export const SHADERS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit render and level-compiler shader libraries",
    group: EToolGroupId.SHADERS,
    help: SHADERS_EDITOR_HELP,
    icon: <PaletteIcon />,
    id: EToolId.SHADERS_EDITOR,
    label: "Shaders editor",
    path: "/shaders-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
