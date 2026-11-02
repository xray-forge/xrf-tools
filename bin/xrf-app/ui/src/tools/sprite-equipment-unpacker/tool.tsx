import { UnarchiveIcon } from "@xrf/ui/icons/unarchive";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const SPRITE_EQUIPMENT_UNPACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Extract individual icons from an equipment sprite",
    group: EToolGroupId.SPRITES,
    icon: <UnarchiveIcon />,
    id: EToolId.SPRITE_EQUIPMENT_UNPACKER,
    label: "Sprite equipment unpacker",
    path: "/sprite-equipment-unpacker",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
