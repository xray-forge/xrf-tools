import { DescriptionIcon } from "@xrf/ui/icons/description";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const SPRITE_DESCRIPTION_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Inspect and edit the icons of a description sprite",
    group: EToolGroupId.SPRITES,
    icon: <DescriptionIcon />,
    id: EToolId.SPRITE_DESCRIPTION_EDITOR,
    label: "Sprite description editor",
    path: "/sprite-description-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
