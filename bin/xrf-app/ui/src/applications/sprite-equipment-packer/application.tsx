import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SPRITE_EQUIPMENT_PACKER_HELP } from "./help";

export const SPRITE_EQUIPMENT_PACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build an equipment sprite from individual icons",
    group: EApplicationGroupId.SPRITES,
    icon: <Inventory2Icon />,
    help: SPRITE_EQUIPMENT_PACKER_HELP,
    id: EApplicationId.SPRITE_EQUIPMENT_PACKER,
    label: "Sprite equipment packer",
    path: "/sprite-equipment-packer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
