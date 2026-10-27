import { CollectionsBookmarkIcon } from "@xrf/ui/icons/collections-bookmark";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { OBJECT_LIBRARY_EDITOR_HELP } from "./help";

export const OBJECT_LIBRARY_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Manage reusable source objects and their previews",
    group: EToolGroupId.VISUALS,
    help: OBJECT_LIBRARY_EDITOR_HELP,
    icon: <CollectionsBookmarkIcon />,
    id: EToolId.OBJECT_LIBRARY_EDITOR,
    label: "Object library editor",
    path: "/object-library-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
