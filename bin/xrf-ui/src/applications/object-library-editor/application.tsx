import { default as CollectionsBookmarkIcon } from "@mui/icons-material/CollectionsBookmark";

import { OBJECT_LIBRARY_EDITOR_HELP } from "@/applications/object-library-editor/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const OBJECT_LIBRARY_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Manage reusable source objects and their previews",
    group: EApplicationGroupId.VISUALS,
    help: OBJECT_LIBRARY_EDITOR_HELP,
    icon: <CollectionsBookmarkIcon />,
    id: EApplicationId.OBJECT_LIBRARY_EDITOR,
    label: "Object library editor",
    path: "/object-library-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
