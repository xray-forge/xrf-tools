import { FilterIcon } from "@xrf/ui/icons/filter";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { POSTPROCESS_EDITOR_HELP } from "./help";

export const POSTPROCESS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Author postprocess effect curves and previews",
    group: EToolGroupId.ENVIRONMENT,
    help: POSTPROCESS_EDITOR_HELP,
    icon: <FilterIcon />,
    id: EToolId.POSTPROCESS_EDITOR,
    label: "Postprocess editor",
    path: "/postprocess-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
