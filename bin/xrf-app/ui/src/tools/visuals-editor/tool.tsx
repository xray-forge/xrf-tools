import { AccessibilityNewIcon } from "@xrf/ui/icons/accessibility-new";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { VISUALS_EDITOR_HELP } from "./help";

export const VISUALS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit model surfaces, skeletons and motion properties",
    group: EToolGroupId.VISUALS,
    help: VISUALS_EDITOR_HELP,
    icon: <AccessibilityNewIcon />,
    id: EToolId.VISUALS_EDITOR,
    label: "Visuals editor",
    path: "/visuals-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
