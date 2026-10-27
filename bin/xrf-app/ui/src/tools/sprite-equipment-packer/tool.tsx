import { Inventory2Icon } from "@xrf/ui/icons/inventory2";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SPRITE_EQUIPMENT_PACKER_HELP } from "./help";

export const SPRITE_EQUIPMENT_PACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build an equipment sprite from individual icons",
    group: EToolGroupId.SPRITES,
    icon: <Inventory2Icon />,
    help: SPRITE_EQUIPMENT_PACKER_HELP,
    id: EToolId.SPRITE_EQUIPMENT_PACKER,
    label: "Sprite equipment packer",
    path: "/sprite-equipment-packer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
