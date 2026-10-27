import { UnarchiveIcon } from "@xrf/ui/icons/unarchive";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const SPRITE_DESCRIPTION_UNPACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Extract individual icons from a description sprite",
    group: EToolGroupId.SPRITES,
    icon: <UnarchiveIcon />,
    id: EToolId.SPRITE_DESCRIPTION_UNPACKER,
    label: "Sprite description unpacker",
    path: "/sprite-description-unpacker",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
